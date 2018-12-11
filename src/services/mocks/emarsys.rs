use std::iter::Iterator;

use models::emarsys::CreateContactRequest;
use services::types::ServiceFuture;
use models::emarsys::DeleteContactResponse;
use models::emarsys::AddToContactListResponse;
use models::emarsys::AddToContactListResponseData;
use services::emarsys::EmarsysClient;
use models::emarsys::EMAIL_FIELD;
use models::emarsys::AddToContactListRequest;
use models::emarsys::CreateContactResponse;
use serde_json::Value as JsonValue;
use models::emarsys::CreateContactResponseData;
use futures::Future;
use serde_json::Map;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct Field {
    key: String,
    value: String,
}

impl Field {
    pub fn new(key: String, value: String) -> Field {
        Field {
            key,
            value,
        }
    }
}

#[derive(Clone)]
pub struct ContactMockData {
    pub key_field: Field,
    pub new_field: Field,
    pub source_id: i64,
}

impl ContactMockData {
    pub fn new(key_field: Field, new_field: Field, source_id: i64) -> ContactMockData {
        ContactMockData {
            key_field,
            new_field,
            source_id,
        }
    }
}

#[derive(Clone)]
pub struct ContactMock {
    id: i64,
    data: ContactMockData,
}

impl ContactMock {
    pub fn new(id: i64, data: ContactMockData) -> ContactMock {
        ContactMock {
            id,
            data,
        }
    }
}

#[derive(Clone)]
pub struct ContactListMock {
    id: i64,
    contacts: Vec<Rc<ContactMock>>,
}

impl ContactListMock {
    pub fn new(id: i64) -> ContactListMock {
        ContactListMock {
            id,
            contacts: vec![],
        }
    }

    pub fn add_contact(&mut self, contact: Rc<ContactMock>) {
        self.contacts.push(contact);
    }
}

#[derive(Clone)]
struct Counter<T: Clone> {
    id: i64,
    value: Vec<Rc<T>>,
}

impl<T: Clone> Counter<T> {
    pub fn new() -> Counter<T> {
        Counter {
            id: 0,
            value: vec![],
        }
    }

    pub fn inc(&mut self) -> i64 {
        let cnt = self.id;
        self.id = cnt + 1;
        cnt
    }

    pub fn push_with_id(&mut self, f: impl FnOnce(i64) -> T) -> Rc<T> {
        let elem = Rc::new(f(self.inc()));
        self.value.push(elem.clone());
        elem
    }

    pub fn push_multiple_with_ids(&mut self, fs: Vec<impl FnOnce(i64) -> T>) -> Vec<Rc<T>> {
        let mut result = vec![];

        for f in fs {
            let elem = Rc::new(f(self.inc()));
            self.value.push(elem.clone());
            result.push(elem);
        }

        result
    }
}

#[derive(Clone)]
struct EmarsysClientMock {
    contacts: RefCell<Counter<ContactMock>>,
    contact_lists: RefCell<Counter<ContactListMock>>,
}

impl EmarsysClientMock {
    pub fn new() -> EmarsysClientMock {
        EmarsysClientMock {
            contacts: RefCell::new(Counter::new()),
            contact_lists: RefCell::new(Counter::new()),
        }
    }

    pub fn create_multiple_contacts(&self, key_id: String, new_contacts: Vec<ContactMockData>) -> Vec<Rc<ContactMock>> {
        let mut contacts = self.contacts.borrow_mut();

        contacts.push_multiple_with_ids(
            new_contacts.iter()
                .map(|data| {
                    let d = data.clone();
                    |id| ContactMock::new(id, d)
                })
                .collect()
        )
    }

    pub fn delete_contacts(&self, email: String) -> Vec<i64> {
        let mut deleted_ids = vec![];
        let mut contacts = self.contacts.borrow_mut();

        contacts.value.retain(|c| {
            let delete = c.data.key_field.key == EMAIL_FIELD && c.data.key_field.value == email;
            if delete { deleted_ids.push(c.id) }
            delete
        });

        deleted_ids
    }

    pub fn create_contact_list(&self) -> Rc<ContactListMock> {
        let mut contact_lists = self.contact_lists.borrow_mut();
        contact_lists.push_with_id(
            |id| ContactListMock::new(id)
        )
    }

    pub fn find_contact_list(&self, contact_list_id: i64) -> Option<Rc<ContactListMock>> {
        let mut contact_lists = self.contact_lists.borrow_mut();

        contact_lists.value.iter_mut()
            .find(|x| x.id == contact_list_id)
            .map(|x| x.clone())
    }

    pub fn find_contacts(&self, key_id: String, external_ids: Vec<String>) -> Vec<Rc<ContactMock>> {
        let mut contacts = self.contacts.borrow_mut();

        contacts.value.iter()
            .filter(|&x| {
                let contact = x.clone();
                contact.data.key_field.key == key_id && external_ids.contains(&contact.data.key_field.value)
            })
            .map(|x| x.clone())
            .collect()
    }
}

impl EmarsysClient for EmarsysClientMock {
    fn add_to_contact_list(
        &self,
        contact_list_id: i64,
        request: AddToContactListRequest,
    ) -> ServiceFuture<AddToContactListResponse> {
        let contacts = self.find_contacts(request.key_id, request.external_ids);
        let contact_list_option = self.find_contact_list(contact_list_id);

        if let Some(mut contact_list) = contact_list_option {
            for contact in contacts {
                Rc::make_mut(&mut contact_list).add_contact(contact.clone())
            }

            Box::new(futures::future::ok(
                AddToContactListResponse {
                    reply_code: Some(0),
                    reply_text: None,       // TODO: return something?
                    data: Some(AddToContactListResponseData {
                        inserted_contacts: Some(contact_list.contacts.len() as i32),
                        errors: None,
                    }),
                }
            ))
        } else {
            // TODO.

            unimplemented!()
        }
    }

    fn create_contact(
        &self,
        request: CreateContactRequest,
    ) -> ServiceFuture<CreateContactResponse> {
        let contacts = self.create_multiple_contacts(request.clone().key_id, request.contacts.iter().map(|v| {
            if let JsonValue::Object(m) = v {
                let mut keys: Vec<String> = vec![];

                for key in m.keys() {
                    keys.push(key.clone());
                }

                let new_field_key_option = keys.iter().find(|&x| {
                    let key = x.clone();
                    key != request.clone().key_id && key != "source_id".to_string()
                });

                let source_id_option = v.get("source_id");
                let key_field_option = v.get(request.clone().key_id);

                // TODO: PLEASE REWRITE ME!!!
                if let Some(new_field_key) = new_field_key_option {
                    if let Some(JsonValue::String(new_field)) = v.get(new_field_key.clone()) {
                        if let Some(JsonValue::Number(source_id)) = source_id_option {
                            if let Some(JsonValue::String(key_field)) = key_field_option {
                                ContactMockData::new(
                                    Field::new(request.clone().key_id, key_field.clone()),
                                    Field::new(new_field_key.clone(), new_field.clone()),

                                    // TODO: rewrite without `.unwrap()`.
                                    source_id.as_i64().unwrap(),
                                )
                            } else { unimplemented!() }
                        } else { unimplemented!() }
                    } else { unimplemented!() }
                } else { unimplemented!() }
            } else {
                unimplemented!()
            }
        }).collect());

        let ids = contacts.iter().map(|c| c.id as i32).collect();

        Box::new(
            futures::future::ok(CreateContactResponse {
                reply_code: Some(0),
                reply_text: None,   // TODO?
                data: Some(CreateContactResponseData {
                    ids: Some(ids),
                    errors: None,
                }),
            })
        )
    }

    fn delete_contact(&self, email: String) -> ServiceFuture<DeleteContactResponse> {
        let ids = self.delete_contacts(email);

        let mut data_map = Map::new();
        data_map.insert("deleted_contacts".to_string(), JsonValue::Number(ids.len().into()));

        Box::new(
            futures::future::ok(DeleteContactResponse {
                reply_code: Some(0),
                reply_text: Some("OK".to_string()),   // TODO?
                data: Some(JsonValue::Object(data_map))
            })
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::emarsys::*;

    static EMAIL_1: &'static str = "bonnie@storiqa.com";
    static EMAIL_2: &'static str = "clyde@storiqa.com";
    static FIRST_NAME_1: &'static str = "Bonnie";
    static FIRST_NAME_2: &'static str = "Clyde";
    static SOURCE_ID_1: i64 = 42;
    static SOURCE_ID_2: i64 = 666;

    fn create_contact_value(
        email: impl Into<String>, first_name: impl Into<String>, source_id: impl Into<i64>
    ) -> JsonValue {
        let json = format!(
            r#"
            {{
                "{email_field}": "{email}",
                "{first_name_field}": "{first_name}",
                "source_id": {source_id}
            }}
            "#,
            email_field=EMAIL_FIELD,
            email=email.into(),
            first_name_field=FIRST_NAME_FIELD,
            first_name=first_name.into(),
            source_id=source_id.into()
        );

        serde_json::from_str(json.as_str()).unwrap()
    }

    fn create_contact_data(
        email: impl Into<String>, first_name: impl Into<String>, source_id: impl Into<i64>
    ) -> ContactMockData {
        ContactMockData::new(
            Field::new(EMAIL_FIELD.into(), email.into()),
            Field::new(FIRST_NAME_FIELD.into(), first_name.into()),
            source_id.into()
        )
    }

    #[test]
    fn create_contact_test() {
        let emarsys = EmarsysClientMock::new();
        let user_data = create_contact_value(
            EMAIL_1,
            FIRST_NAME_1,
            SOURCE_ID_1
        );
        let request = CreateContactRequest {
            key_id: EMAIL_FIELD.to_string(),
            contacts: vec![
                user_data
            ],
        };
        let response = emarsys.create_contact(request).wait();
//        panic!("::: {:?} :::", response);
        // TODO: check response.
    }

    #[test]
    fn delete_contact_test() {
        let emarsys = EmarsysClientMock::new();
        let contacts = vec![
            create_contact_data(EMAIL_1, FIRST_NAME_1, SOURCE_ID_1),
            create_contact_data(EMAIL_2, FIRST_NAME_2, SOURCE_ID_2)
        ];
        emarsys.create_multiple_contacts(EMAIL_FIELD.to_string(), contacts);

        let response = emarsys.delete_contact(EMAIL_1.into()).wait()
            .expect("API request failed");

        let data = response.data.clone().expect("Response `data` field is missing");
        if let JsonValue::Object(map) = data {
            let deleted_contacts = map.get("deleted_contacts")
                .expect("Response `data.deleted_contacts` field is missing");
            assert_eq!(deleted_contacts, 1);
        } else {
            panic!("Response `data` field is not an object");
        }
    }

    #[test]
    fn add_contact_to_contact_list_test() {
        let emarsys = EmarsysClientMock::new();
        let contacts = vec![
            create_contact_data(EMAIL_1, FIRST_NAME_1, SOURCE_ID_1),
            create_contact_data(EMAIL_2, FIRST_NAME_2, SOURCE_ID_2)
        ];
        emarsys.create_multiple_contacts(EMAIL_FIELD.to_string(), contacts.clone());

        let contact_list_id = emarsys.create_contact_list().id;

        let request = AddToContactListRequest {
            key_id: EMAIL_FIELD.into(),
            external_ids: vec![EMAIL_1.into(), EMAIL_2.into()]
        };
        let response = emarsys.add_to_contact_list(contact_list_id, request).wait()
            .expect("API request failed");

        let data = response.data.clone()
            .expect("Response `data` field is missing");

        let inserted_contacts = data.inserted_contacts
            .expect("Response `data.deleted_contacts` field is missing");
        assert_eq!(inserted_contacts, contacts.len() as i32);
    }
}