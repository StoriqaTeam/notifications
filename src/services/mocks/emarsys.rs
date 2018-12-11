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
    key_field: Field,
    new_field: Field,
    source_id: i64,
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
    contacts: Vec<ContactMock>,
}

impl ContactListMock {
    pub fn new(id: i64) -> ContactListMock {
        ContactListMock {
            id,
            contacts: vec![],
        }
    }

    pub fn add_contact(&mut self, contact: ContactMock) {
        self.contacts.push(contact);
    }
}

#[derive(Clone)]
struct Counter<T: Clone> {
    id: i64,
    value: Vec<T>,
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

    pub fn push_with_id(mut self, f: impl FnOnce(i64) -> T) -> T {
        let elem = f(self.inc());
        self.value.push(elem.clone());
        elem
    }

    pub fn push_multiple_with_ids(mut self, fs: Vec<impl FnOnce(i64) -> T>) -> Vec<T> {
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
struct EmarsysClientMock {
    contacts: Counter<ContactMock>,
    contact_lists: Counter<ContactListMock>,
}

impl EmarsysClientMock {
    pub fn new() -> EmarsysClientMock {
        EmarsysClientMock {
            contacts: Counter::new(),
            contact_lists: Counter::new(),
        }
    }

    pub fn create_multiple_contacts(self, key_id: String, contacts: Vec<ContactMockData>) -> Vec<ContactMock> {
        self.contacts.push_multiple_with_ids(
            contacts.iter()
                .map(|data| {
                    let d = data.clone();
                    |id| ContactMock::new(id, d)
                })
                .collect()
        )
    }

    pub fn delete_contacts(mut self, email: String) -> Vec<i64> {
        let mut deleted_ids = vec![];
        self.contacts.value.retain(|c| {
            let delete = c.data.key_field.key == EMAIL_FIELD && c.data.key_field.value == email;
            if delete { deleted_ids.push(c.id) }
            delete
        });

        deleted_ids
    }

    pub fn create_contact_list(self) -> ContactListMock {
        self.contact_lists.push_with_id(|id| ContactListMock::new(id))
    }

    pub fn find_contact_list(&mut self, contact_list_id: i64) -> Option<&mut ContactListMock> {
        self.contact_lists.value.iter_mut()
            .find(|x| x.id == contact_list_id)
    }

    pub fn find_contacts(&self, key_id: String, external_ids: Vec<String>) -> Vec<ContactMock> {
        self.contacts.value.iter()
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
        mut self,
        contact_list_id: i64,
        request: AddToContactListRequest,
    ) -> ServiceFuture<AddToContactListResponse> {
        let contacts = self.find_contacts(request.key_id, request.external_ids);
        let mut contact_list_option = self.find_contact_list(contact_list_id);

        if let Some(mut contact_list) = contact_list_option {
            for contact in contacts {
                contact_list.add_contact(contact.clone())
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
        self,
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

    fn delete_contact(self, email: String) -> ServiceFuture<DeleteContactResponse> {
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
    use models::emarsys::CreateContactRequest;
    use super::*;

    #[test]
    fn create_contact() {
        let emarsys = EmarsysClientMock::new();
        let user_data = serde_json::from_str(
            r#"
            {
                "3": "foo@bar.baz",
                "1": "John",
                "source_id": 42
            }
            "#).unwrap();   // TODO: without unwrap.
        let request = CreateContactRequest {
            key_id: EMAIL_FIELD.to_string(),
            contacts: vec![
                user_data
            ],
        };
        let response = emarsys.create_contact(request).wait();
        panic!("::: {:?} :::", response);
        // TODO: check response.
    }

//    #[test]
//    fn delete_contact() {
//        let emarsys: EmarsysClient = EmarsysClientMock::new();
//        let request = ();
//        let response = block_on(emarsys.delete_contact(request));
//        // TODO: check response.
//    }

//    #[test]
//    fn add_contact_to_contact_list() {
//        let emarsys = EmarsysClientMock::new();
//        let contact_list_id = emarsys.create_contact_list().id;
//        let emarsys = emarsys as EmarsysClient;
//        let request = ();
//        let response = block_on(emarsys.add_to_contact_list(request));
//        // TODO: check response.
//    }
}