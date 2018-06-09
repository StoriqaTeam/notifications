use failure::Error as FailureError;
use failure::Fail;
use futures::prelude::*;
use futures_cpupool::CpuPool;
use hyper::header::{Authorization, Bearer, ContentType};
use hyper::{mime, Headers, Method};
use serde_json;

use stq_http::client::ClientHandle;
use stq_http::client::Error as HttpError;

use super::types::ServiceFuture;
use config::SendGridConf;
use errors::Error;
use models::sendgrid::from_simple_mail;
use models::SimpleMail;

pub trait MailService {
    /// Send simple mail
    fn send_mail(&self, mail: SimpleMail) -> ServiceFuture<String>;
}

/// SendGrid service implementation
pub struct SendGridServiceImpl {
    pub cpu_pool: CpuPool,
    pub http_client: ClientHandle,
    pub send_grid_conf: SendGridConf,
}

impl SendGridServiceImpl {
    pub fn new(cpu_pool: CpuPool, http_client: ClientHandle, send_grid_conf: SendGridConf) -> Self {
        Self {
            cpu_pool,
            http_client,
            send_grid_conf,
        }
    }
}

impl MailService for SendGridServiceImpl {
    fn send_mail(&self, mail: SimpleMail) -> ServiceFuture<String> {
        let http_clone = self.http_client.clone();
        let config = self.send_grid_conf.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    let url = format!("{}/{}", config.api_addr.clone(), config.send_mail_path.clone());

                    let payload = from_simple_mail(mail, config.from_email.clone());
                    serde_json::to_string(&payload)
                        .into_future()
                        .map_err(|e| e.context("Couldn't parse payload").into())
                        .and_then(move |body| {
                            info!("Sending payload: {}", &body);

                            let mut headers = Headers::new();
                            let api_key = config.api_key.clone();
                            headers.set(Authorization(Bearer { token: api_key }));
                            headers.set(ContentType(mime::APPLICATION_JSON));

                            http_clone
                                .request::<String>(Method::Post, url, Some(body), Some(headers))
                                .or_else(|e| {
                                    // Required due to problem of parsing empty body
                                    match e {
                                        HttpError::Parse(_) => Ok("Ok".to_string()),
                                        error => Err(error.context(Error::HttpClient).into()),
                                    }
                                })
                                .map(|_| "Ok".to_string())
                        })
                })
                .map_err(|e: FailureError| e.context("Mail service, send_mail endpoint error occured.").into()),
        )
    }
}
