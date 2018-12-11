use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Fail;
use futures::{Future, IntoFuture};
use hyper::header::ContentType;
use hyper::{mime, Headers, Method};
use r2d2::ManageConnection;

use stq_http::client::ClientHandle;
use stq_http::request_util::XWSSE;

use config::EmarsysConf;
use errors::Error;
use models::{
    AddToContactListRequest, AddToContactListResponse, CreateContactPayload, CreateContactRequest, CreateContactResponse, CreatedContact,
    DeleteContactPayload, DeleteContactResponse, Signature, EMAIL_FIELD,
};
use repos::ReposFactory;
use services::types::{Service, ServiceFuture};
use models::emarsys::AddToContactListResponseData;

pub trait EmarsysService {
    fn emarsys_create_contact(&self, payload: CreateContactPayload) -> ServiceFuture<CreatedContact>;
    fn emarsys_delete_contact(&self, payload: DeleteContactPayload) -> ServiceFuture<()>;
}

impl<T, M, F> EmarsysService for Service<T, M, F>
where
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
{
    fn emarsys_delete_contact(&self, payload: DeleteContactPayload) -> ServiceFuture<()> {
        info!("deleting user {} from emarsys", payload.user_id);
        let http_clone = self.static_context.client_handle.clone();
        let user_email = payload.email;
        let res = self
            .static_context
            .config
            .emarsys
            .clone()
            .ok_or(format_err!("Emarsys config not found"))
            .into_future()
            .map(move |emarsys_conf| EmarsysClientImpl {
                config: emarsys_conf,
                client_handle: http_clone,
            }).and_then(move |emarsys_client| emarsys_client.delete_contact(user_email))
            .and_then(|response| response.into_result());

        Box::new(res)
    }

    fn emarsys_create_contact(&self, payload: CreateContactPayload) -> ServiceFuture<CreatedContact> {
        let user_id = payload.user_id;
        info!("sending user {}, email: {} to emarsys", user_id, payload.email);
        let http_clone = self.static_context.client_handle.clone();
        let user_id = payload.user_id;
        let user_email = payload.email.clone();
        let res = self
            .static_context
            .config
            .emarsys
            .clone()
            .ok_or(format_err!("Emarsys config not found for user {}", user_id))
            .into_future()
            .map(move |emarsys_conf| EmarsysClientImpl {
                config: emarsys_conf,
                client_handle: http_clone,
            }).and_then(|emarsys_client| {
                let request = CreateContactRequest::from(payload);
                emarsys_client.clone().create_contact(request).map(|r| (emarsys_client, r))
            }).and_then(move |(emarsys_client, response)| {
                response
                    .extract_cteated_id()
                    .map_err(|e| {
                        e.context(format_err!(
                            "Emarsys for user {} error in response. Response: {:?}",
                            user_id,
                            response
                        )).into()
                    }).map(|id| (emarsys_client, id))
            }).and_then(move |(emarsys_client, emarsys_id)| {
                info!("Emarsys create contact for {}, trying to add it to contact list", user_id);
                let request = AddToContactListRequest::from_email(user_email);
                let contact_list_id = emarsys_client.config.registration_contact_list_id;
                emarsys_client
                    .add_to_contact_list(contact_list_id, request)
                    .map(|response| {
                        let inserted_contacts = response.extract_inserted_contacts();
                        (response, inserted_contacts)
                    }).then(move |res| {
                        match res {
                            Ok((_response, Ok(inserted_contacts))) => {
                                info!(
                                    "Emarsys for user {} added {} contact(s) to contact list",
                                    user_id, inserted_contacts
                                );
                            }
                            Ok((response, Err(error))) => {
                                error!(
                                    "Emarsys for user {} something happend during add to contact list: {}, response: {:?}",
                                    user_id, error, response
                                );
                            }
                            Err(error) => {
                                error!("Error for user {} during add to contact list: {:?}", user_id, error);
                            }
                        }
                        Ok(emarsys_id)
                    })
            }).then(|res| match res {
                Ok(id) => Ok(id),
                Err(err) => {
                    error!("{}", err);
                    Err(err)
                }
            }).map(move |emarsys_id| CreatedContact { emarsys_id, user_id });
        Box::new(res)
    }
}

pub trait EmarsysClient {
    fn add_to_contact_list(
        &self,
        contact_list_id: i64,
        request: AddToContactListRequest
    ) -> ServiceFuture<AddToContactListResponse>;

    fn create_contact(
        &self,
        request: CreateContactRequest
    ) -> ServiceFuture<CreateContactResponse>;

    fn delete_contact(
        &self,
        email: String
    ) -> ServiceFuture<DeleteContactResponse>;
}

#[derive(Clone)]
struct EmarsysClientImpl {
    config: EmarsysConf,
    client_handle: ClientHandle,
}

impl EmarsysClient for EmarsysClientImpl {
    fn add_to_contact_list(
        &self,
        contact_list_id: i64,
        request: AddToContactListRequest
    ) -> ServiceFuture<AddToContactListResponse> {
        let config = self.config.clone();
        let signature = Signature::new(config.username_token, config.api_secret_key);
        let url = format!("{}/contactlist/{}/add", self.config.api_addr, contact_list_id);

        debug!(
            "EmarsysClient add_to_contact_list: url=\"{}\"; signature: {:?}; request: {:?}",
            url, signature, request
        );

        let mut headers = Headers::new();
        headers.set(ContentType(mime::APPLICATION_JSON));
        let xwsse: XWSSE = signature.into();
        headers.set(xwsse);

        let client_handle = self.client_handle.clone();
        Box::new(serde_json::to_string(&request)
            .into_future()
            .map_err(|e| e.context("Couldn't serialize payload").into())
            .and_then(move |request_body| {
                client_handle
                    .request::<AddToContactListResponse>(Method::Post, url, Some(request_body), Some(headers))
                    .map_err(|e| e.context(Error::HttpClient).into())
            }))
    }

    fn create_contact(&self, request: CreateContactRequest) -> ServiceFuture<CreateContactResponse> {
        let config = self.config.clone();
        let signature = Signature::new(config.username_token, config.api_secret_key);
        let url = format!("{}/contact", self.config.api_addr);

        debug!(
            "EmarsysClient create_contact: url=\"{}\"; signature: {:?}; request: {:?}",
            url, signature, request
        );

        let mut headers = Headers::new();
        headers.set(ContentType(mime::APPLICATION_JSON));
        let xwsse: XWSSE = signature.into();
        headers.set(xwsse);

        let client_handle = self.client_handle.clone();
        Box::new(serde_json::to_string(&request)
            .into_future()
            .map_err(|e| e.context("Couldn't serialize payload").into())
            .and_then(move |request_body| {
                client_handle
                    .request::<CreateContactResponse>(Method::Post, url, Some(request_body), Some(headers))
                    .map_err(|e| e.context(Error::HttpClient).into())
            }))
    }

    fn delete_contact(&self, email: String) -> ServiceFuture<DeleteContactResponse> {
        let config = self.config.clone();
        let signature = Signature::new(config.username_token, config.api_secret_key);
        let url = format!("{}/contact/delete", self.config.api_addr);

        let request = serde_json::json!({ EMAIL_FIELD: email });

        debug!(
            "EmarsysClient delete_contact: url=\"{}\"; signature: {:?}; request: {:?}",
            url, signature, request
        );

        let mut headers = Headers::new();
        headers.set(ContentType(mime::APPLICATION_JSON));
        let xwsse: XWSSE = signature.into();
        headers.set(xwsse);

        let client_handle = self.client_handle.clone();
        Box::new(serde_json::to_string(&request)
            .into_future()
            .map_err(|e| e.context("Couldn't serialize payload").into())
            .and_then(move |request_body| {
                client_handle
                    .request::<DeleteContactResponse>(Method::Post, url, Some(request_body), Some(headers))
                    .map_err(|e| e.context(Error::HttpClient).into())
            }))
    }
}