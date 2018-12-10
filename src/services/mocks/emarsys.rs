use models::emarsys::CreateContactRequest;
use services::types::ServiceFuture;
use models::emarsys::DeleteContactResponse;
use models::emarsys::AddToContactListResponse;
use models::emarsys::AddToContactListResponseData;
use services::emarsys::EmarsysClient;
use models::emarsys::EMAIL_FIELD;
use models::emarsys::AddToContactListRequest;
use models::emarsys::CreateContactResponse;

#[derive(Clone)]
pub struct ContactMock {
    id: i64,
    email: String
}

impl ContactMock {
    pub fn new(id: i64, email: String) -> ContactMock {
        ContactMock {
            id,
            email
        }
    }
}

#[derive(Clone)]
pub struct ContactListMock {
    id: i64,
    contacts: Vec<ContactMock>
}

impl ContactListMock {
    pub fn new(id: i64) -> ContactListMock {
        ContactListMock {
            id,
            contacts: vec![]
        }
    }

    pub fn add_contact(&mut self, contact: ContactMock) {
        self.contacts.push(contact);
    }
}

#[derive(Clone)]
struct Counter<T: Clone> {
    id: i64,
    value: Vec<T>
}

impl<T: Clone> Counter<T> {
    pub fn new() -> Counter<T> {
        Counter {
            id: 0,
            value: vec![]
        }
    }

    pub fn inc(&mut self) -> i64 {
        let cnt = self.id;
        self.id = cnt + 1;
        cnt
    }

    pub fn push_with_id(mut self, f: impl Fn(i64) -> T) -> T {
        let elem = f(self.inc());
        self.value.push(elem.clone());
        elem
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
            contact_lists: Counter::new()
        }
    }

    pub fn create_contact(self, email: String) -> ContactMock {
        self.contacts.push_with_id(|id| ContactMock::new(id, email.clone()))
    }

    pub fn create_contact_list(self) -> ContactListMock {
        self.contact_lists.push_with_id(|id| ContactListMock::new(id))
    }

    pub fn find_contact_list(&mut self, contact_list_id: i64) -> Option<&mut ContactListMock> {
        self.contact_lists.value.iter_mut()
            .find(|x| x.id == contact_list_id)
//            .map(|x| x.clone())
    }

    pub fn find_contacts(&self, key_id: String, external_ids: Vec<String>) -> Vec<ContactMock> {
        if key_id == EMAIL_FIELD {
            self.contacts.value.iter()
                .filter(|x| external_ids.contains(&x.email))
                .map(|x| x.clone())
                .collect()
        } else {
            unimplemented!()
        }
    }
}

impl EmarsysClient for EmarsysClientMock {
    fn add_to_contact_list(
        mut self,
        contact_list_id: i64,
        request: AddToContactListRequest
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
                        errors: None
                    })
                }
            ))
        } else {
            // TODO.

            unimplemented!()
        }
    }

    fn create_contact(self, _request: CreateContactRequest) -> ServiceFuture<CreateContactResponse> {
        unimplemented!()
    }

    fn delete_contact(self, _email: String) -> ServiceFuture<DeleteContactResponse> {
        unimplemented!()
    }
}