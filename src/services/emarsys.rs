use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Fail;
use futures::{Future, IntoFuture};
use hyper::header::ContentType;
use hyper::{mime, Headers, Method};
use r2d2::ManageConnection;

use stq_http::request_util::XWSSE;

use errors::Error;
use models::{CreateContactPayload, CreateContactRequest, CreateContactResponse, CreatedContact, Signature};
use repos::ReposFactory;
use services::types::{Service, ServiceFuture};

pub trait EmarsysService {
    fn emarsys_create_contact(&self, payload: CreateContactPayload) -> ServiceFuture<CreatedContact>;
}

impl<T, M, F> EmarsysService for Service<T, M, F>
where
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
{
    fn emarsys_create_contact(&self, payload: CreateContactPayload) -> ServiceFuture<CreatedContact> {
        info!("sending user {} to emarsys", payload.user_id);
        let http_clone = self.static_context.client_handle.clone();
        let user_id = payload.user_id;
        let res = self
            .static_context
            .config
            .emarsys
            .clone()
            .ok_or(format_err!("Emarsys config not found"))
            .into_future()
            .map(|emarsys_conf| {
                let signature = Signature::new(emarsys_conf.username_token, emarsys_conf.api_secret_key);
                let request = CreateContactRequest::from(payload);
                let url = format!("{}/contact", emarsys_conf.api_addr);
                (url, signature, request)
            }).inspect(|(url, signature, request)| {
                debug!(
                    "emarsys_create_contact: url=\"{}\"; signature: {:?}; request: {:?}",
                    url, signature, request
                );
            }).and_then(|(url, signature, request)| {
                serde_json::to_string(&request)
                    .map_err(|e| e.context("Couldn't serialize payload").into())
                    .map(|request_body| (url, signature, request_body))
            }).and_then(move |(url, signature, request_body)| {
                let mut headers = Headers::new();
                headers.set(ContentType(mime::APPLICATION_JSON));
                let xwsse: XWSSE = signature.into();
                headers.set(xwsse);

                http_clone
                    .request::<CreateContactResponse>(Method::Post, url, Some(request_body), Some(headers))
                    .map_err(|e| e.context(Error::HttpClient).into())
            }).and_then(|response| {
                response
                    .extract_cteated_id()
                    .map_err(|e| e.context(format_err!("Error in emarsys response. Response: {:?}", response)).into())
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
