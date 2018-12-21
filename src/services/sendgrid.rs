use failure::Fail;
use futures::prelude::*;
use hyper::header::{Authorization, Bearer, ContentType};
use hyper::{mime, Headers, Method};

use stq_http::client::ClientHandle;

use config::SendGridConf;
use errors::Error;
use models::SendGridPayload;
use services::types::ServiceFuture;

pub trait SendgridService: Send + Sync {
    fn send(&self, payload: SendGridPayload) -> ServiceFuture<()>;
}

pub struct SendgridServiceImpl {
    pub config: SendGridConf,
    pub client_handle: ClientHandle,
}

impl SendgridService for SendgridServiceImpl {
    fn send(&self, payload: SendGridPayload) -> ServiceFuture<()> {
        let SendGridConf {
            api_addr,
            api_key,
            send_mail_path,
            ..
        } = self.config.clone();
        let url = format!("{}/{}", api_addr.clone(), send_mail_path.clone());

        let mut headers = Headers::new();
        headers.set(Authorization(Bearer { token: api_key }));
        headers.set(ContentType(mime::APPLICATION_JSON));

        let client_handle = self.client_handle.clone();

        let res = serde_json::to_string(&payload)
            .into_future()
            .map_err(|e| e.context("Couldn't parse payload").into())
            .and_then(move |body| {
                client_handle
                    .request::<()>(Method::Post, url, Some(body), Some(headers))
                    .map_err(|e| e.context(Error::HttpClient).into())
            });
        Box::new(res)
    }
}
