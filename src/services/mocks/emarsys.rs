use std::iter::Iterator;

use models::emarsys::AddToContactListRequest;
use models::emarsys::AddToContactListResponse;
use models::emarsys::AddToContactListResponseData;
use models::emarsys::CreateContactRequest;
use models::emarsys::CreateContactResponse;
use models::emarsys::CreateContactResponseData;
use models::emarsys::DeleteContactResponse;
use models::emarsys::EMAIL_FIELD;
use serde_json::Map;
use serde_json::Value as JsonValue;
use services::emarsys::EmarsysClient;
use services::types::ServiceFuture;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone)]
pub struct ContactMockData {
    pub key_id: String,
    pub fields: HashMap<String, String>,
    pub source_id: i64,
}

impl ContactMockData {
    pub fn new(key_id: String, fields: HashMap<String, String>, source_id: i64) -> ContactMockData {
        ContactMockData { key_id, fields, source_id }
    }
}

#[derive(Clone)]
pub struct ContactMock {
    id: i64,
    data: ContactMockData,
}

impl ContactMock {
    pub fn new(id: i64, data: ContactMockData) -> ContactMock {
        ContactMock { id, data }
    }
}

#[derive(Clone)]
pub struct ContactListMock {
    pub id: i64,
    pub contacts: Vec<ContactMock>,
}

impl ContactListMock {
    pub fn new(id: i64) -> ContactListMock {
        ContactListMock { id, contacts: vec![] }
    }

    pub fn add_contact(&mut self, contact: ContactMock) {
        self.contacts.push(contact);
    }
}

#[derive(Clone)]
pub struct Counter<T: Clone> {
    pub id: i64,
    pub value: Vec<T>,
}

impl<T: Clone> Counter<T> {
    pub fn new() -> Counter<T> {
        Counter { id: 0, value: vec![] }
    }

    pub fn inc(&mut self) -> i64 {
        let cnt = self.id;
        self.id = cnt + 1;
        cnt
    }

    pub fn push_with_id(&mut self, f: impl FnOnce(i64) -> T) -> T {
        let elem = f(self.inc());
        self.value.push(elem.clone());
        elem
    }

    pub fn push_multiple_with_ids(&mut self, fs: Vec<impl FnOnce(i64) -> T>) -> Vec<T> {
        let mut result = vec![];

        for f in fs {
            let elem = f(self.inc());
            self.value.push(elem.clone());
            result.push(elem);
        }

        result
    }
}

#[derive(Clone)]
pub struct EmarsysClientMockState {
    pub contacts: Counter<ContactMock>,
    pub contact_lists: Counter<ContactListMock>,
}

#[derive(Clone)]
pub struct EmarsysClientMock {
    pub state: Arc<Mutex<EmarsysClientMockState>>,
}

impl EmarsysClientMock {
    pub fn new() -> EmarsysClientMock {
        EmarsysClientMock {
            state: Arc::new(Mutex::new(EmarsysClientMockState {
                contacts: Counter::new(),
                contact_lists: Counter::new(),
            })),
        }
    }

    pub fn create_multiple_contacts(&self, new_contacts: Vec<ContactMockData>) -> Vec<ContactMock> {
        let mut state = self.state.lock().unwrap();
        let ref mut contacts = state.contacts;

        contacts.push_multiple_with_ids(
            new_contacts
                .iter()
                .map(|data| {
                    let data = data.clone();
                    |id| ContactMock::new(id, data)
                })
                .collect(),
        )
    }

    pub fn delete_contacts(&self, email: String) -> Vec<i64> {
        let mut state = self.state.lock().unwrap();

        let mut deleted_ids = vec![];
        let ref mut contacts = state.contacts;

        contacts.value.retain(|contact| {
            let delete = contact.data.key_id == EMAIL_FIELD && contact.data.fields.get(EMAIL_FIELD.into()) == Some(&email);
            if delete {
                deleted_ids.push(contact.id)
            }
            delete
        });

        deleted_ids
    }

    pub fn create_contact_list(&self) -> ContactListMock {
        let mut state = self.state.lock().unwrap();

        let ref mut contact_lists = state.contact_lists;
        contact_lists.push_with_id(|id| ContactListMock::new(id))
    }

    pub fn find_contacts(&self, key_id: String, external_ids: Vec<String>) -> Vec<ContactMock> {
        let state = self.state.lock().unwrap();
        let ref contacts = state.contacts;

        contacts
            .value
            .iter()
            .filter(|&contact| contact.data.key_id == key_id && external_ids.contains(&contact.data.fields[&key_id]))
            .map(|x| x.clone())
            .collect()
    }
}

impl EmarsysClient for EmarsysClientMock {
    fn add_to_contact_list(&self, contact_list_id: i64, request: AddToContactListRequest) -> ServiceFuture<AddToContactListResponse> {
        let contacts = self.find_contacts(request.key_id, request.external_ids);
        let ref mut state = self.state.lock().unwrap();

        let mut found_contact_list = false;
        for contact_list in &mut state.contact_lists.value {
            if contact_list.id == contact_list_id {
                found_contact_list = true;
                for contact in contacts.clone() {
                    contact_list.contacts.push(contact);
                }
                break;
            }
        }

        if found_contact_list {
            Box::new(futures::future::ok(AddToContactListResponse {
                reply_code: Some(0),
                reply_text: Some("OK".to_string()),
                data: Some(AddToContactListResponseData {
                    inserted_contacts: Some(contacts.len() as i32),
                    errors: None,
                }),
            }))
        } else {
            Box::new(futures::future::ok(AddToContactListResponse {
                reply_code: Some(1008),
                reply_text: Some("Contact list does not exist".to_string()),
                data: None,
            }))
        }
    }

    fn create_contact(&self, request: CreateContactRequest) -> ServiceFuture<CreateContactResponse> {
        let mut contacts_data = vec![];

        for contact_value in request.contacts.clone() {
            let contact_object = contact_value.as_object();
            if contact_object.is_none() {
                return Box::new(futures::future::ok(CreateContactResponse {
                    reply_code: Some(10001), // TODO: Find proper error code.
                    reply_text: Some("Contact data should be an object".to_string()),
                    data: None,
                }));
            }
            let contact_object = contact_object.unwrap();

            let mut keys = vec![];

            for key in contact_object.keys() {
                keys.push(key.clone())
            }

            let key_id = request.key_id.clone();

            let new_field_key = keys.iter().find(|&key| {
                let key = key.clone();
                key != key_id.clone() && key != "source_id".to_string()
            });
            if new_field_key.is_none() {
                return Box::new(futures::future::ok(CreateContactResponse {
                    reply_code: Some(2005),
                    reply_text: Some("No data provided for key field".to_string()),
                    data: None,
                }));
            }
            let new_field_key = new_field_key.unwrap();

            let new_field_value = contact_value.get(new_field_key);
            if new_field_value.is_none() {
                return Box::new(futures::future::ok(CreateContactResponse {
                    reply_code: Some(2005),
                    reply_text: Some("No data provided for key field".to_string()),
                    data: None,
                }));
            }
            let new_field_value = new_field_value.unwrap().as_str();
            if new_field_value.is_none() {
                return Box::new(futures::future::ok(CreateContactResponse {
                    reply_code: Some(10001),
                    reply_text: Some("Key field value should be a string".to_string()),
                    data: None,
                }));
            }
            let new_field_value = new_field_value.unwrap().to_string();

            let source_id = contact_value.get("source_id");
            if source_id.is_none() {
                return Box::new(futures::future::ok(CreateContactResponse {
                    reply_code: Some(2013),
                    reply_text: Some("No source ID provided".to_string()),
                    data: None,
                }));
            }
            let source_id = source_id.unwrap().as_i64();
            if source_id.is_none() {
                return Box::new(futures::future::ok(CreateContactResponse {
                    reply_code: Some(10001),
                    reply_text: Some("Source ID value should be a string".to_string()),
                    data: None,
                }));
            }
            let source_id = source_id.unwrap();

            let key_field_value = contact_value.get(key_id.clone());
            if key_field_value.is_none() {
                return Box::new(futures::future::ok(CreateContactResponse {
                    reply_code: Some(2005),
                    reply_text: Some(format!("No value provided for key field: {}", key_id.clone())),
                    data: None,
                }));
            }
            let key_field_value = key_field_value.unwrap().as_str();
            if key_field_value.is_none() {
                return Box::new(futures::future::ok(CreateContactResponse {
                    reply_code: Some(10001),
                    reply_text: Some("Key field value should be a string".to_string()),
                    data: None,
                }));
            }
            let key_field_value = key_field_value.unwrap().to_string();

            let mut fields = HashMap::new();
            fields.insert(key_id.clone(), key_field_value.clone());
            fields.insert(new_field_key.clone(), new_field_value.clone());

            contacts_data.push(ContactMockData::new(key_id, fields, source_id));
        }

        let contacts = self.create_multiple_contacts(contacts_data);

        let ids = contacts.iter().map(|c| c.id as i32).collect();

        Box::new(futures::future::ok(CreateContactResponse {
            reply_code: Some(0),
            reply_text: Some("OK".to_string()),
            data: Some(CreateContactResponseData {
                ids: Some(ids),
                errors: None,
            }),
        }))
    }

    fn delete_contact(&self, email: String) -> ServiceFuture<DeleteContactResponse> {
        let ids = self.delete_contacts(email);

        let mut data_map = Map::new();
        data_map.insert("deleted_contacts".to_string(), JsonValue::Number(ids.len().into()));

        Box::new(futures::future::ok(DeleteContactResponse {
            reply_code: Some(0),
            reply_text: Some("OK".to_string()),
            data: Some(JsonValue::Object(data_map)),
        }))
    }
}

#[cfg(test)]
mod tests {
    use futures::Future;

    use super::*;
    use models::emarsys::*;

    static EMAIL_1: &'static str = "bonnie@storiqa.com";
    static EMAIL_2: &'static str = "clyde@storiqa.com";
    static FIRST_NAME_1: &'static str = "Bonnie";
    static FIRST_NAME_2: &'static str = "Clyde";
    static SOURCE_ID_1: i64 = 42;
    static SOURCE_ID_2: i64 = 666;

    fn create_contact_value(email: impl Into<String>, first_name: impl Into<String>, source_id: impl Into<i64>) -> JsonValue {
        let json = format!(
            r#"
            {{
                "{email_field}": "{email}",
                "{first_name_field}": "{first_name}",
                "source_id": {source_id}
            }}
            "#,
            email_field = EMAIL_FIELD,
            email = email.into(),
            first_name_field = FIRST_NAME_FIELD,
            first_name = first_name.into(),
            source_id = source_id.into()
        );

        serde_json::from_str(json.as_str()).unwrap()
    }

    fn create_contact_data(email: impl Into<String>, first_name: impl Into<String>, source_id: impl Into<i64>) -> ContactMockData {
        let mut fields = HashMap::new();
        fields.insert(EMAIL_FIELD.into(), email.into());
        fields.insert(FIRST_NAME_FIELD.into(), first_name.into());
        ContactMockData::new(EMAIL_FIELD.into(), fields, source_id.into())
    }

    #[test]
    fn test_create_contact() {
        let emarsys = EmarsysClientMock::new();
        let user_data = create_contact_value(EMAIL_1, FIRST_NAME_1, SOURCE_ID_1);
        let request = CreateContactRequest {
            key_id: EMAIL_FIELD.to_string(),
            contacts: vec![user_data],
        };
        let response = emarsys.create_contact(request).wait().expect("API request failed");
        assert_eq!(response.reply_code, Some(0));

        let data = response.data.clone().expect("Response `data` field is missing");
        assert_eq!(data.ids.map(|x| x.len()).unwrap_or(0), 1);
        assert_eq!(data.errors, None);
    }

    #[test]
    fn test_delete_contact() {
        let emarsys = EmarsysClientMock::new();
        let contacts = vec![
            create_contact_data(EMAIL_1, FIRST_NAME_1, SOURCE_ID_1),
            create_contact_data(EMAIL_2, FIRST_NAME_2, SOURCE_ID_2),
        ];
        emarsys.create_multiple_contacts(contacts);

        let response = emarsys.delete_contact(EMAIL_1.into()).wait().expect("API request failed");
        assert_eq!(response.reply_code, Some(0));

        let data = response.data.clone().expect("Response `data` field is missing");
        if let JsonValue::Object(map) = data {
            let deleted_contacts = map
                .get("deleted_contacts")
                .expect("Response `data.deleted_contacts` field is missing");
            assert_eq!(deleted_contacts, 1);
        } else {
            panic!("Response `data` field is not an object");
        }
    }

    #[test]
    fn test_add_contact_to_contact_list() {
        let emarsys = EmarsysClientMock::new();
        let contacts = vec![
            create_contact_data(EMAIL_1, FIRST_NAME_1, SOURCE_ID_1),
            create_contact_data(EMAIL_2, FIRST_NAME_2, SOURCE_ID_2),
        ];
        emarsys.create_multiple_contacts(contacts.clone());

        let contact_list = emarsys.create_contact_list();
        let contact_list_id = contact_list.id;

        let request = AddToContactListRequest {
            key_id: EMAIL_FIELD.into(),
            external_ids: vec![EMAIL_1.into(), EMAIL_2.into()],
        };
        let response = emarsys
            .add_to_contact_list(contact_list_id, request.clone())
            .wait()
            .expect("API request failed");
        emarsys
            .add_to_contact_list(contact_list_id, request)
            .wait()
            .expect("API request failed");
        assert_eq!(response.reply_code, Some(0));

        let data = response.data.clone().expect("Response `data` field is missing");

        let inserted_contacts = data.inserted_contacts.expect("Response `data.deleted_contacts` field is missing");
        assert_eq!(inserted_contacts, contacts.len() as i32);

        let state = emarsys.state.lock().unwrap();
        let contact_list = state.contact_lists.value.first().unwrap();
        assert_eq!(contact_list.contacts.len(), 4);
    }
}
