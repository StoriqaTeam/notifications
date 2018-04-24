use futures::prelude::*;
use futures_cpupool::CpuPool;
use hyper::header::{Authorization, Bearer, ContentType};
use hyper::{mime, Headers, Method};
use serde_json;

use stq_http::client::ClientHandle;

use models::SimpleMail;
use config::SendGridConf;
use super::types::ServiceFuture;
use super::error::ServiceError;

use models::sendgrid::from_simple_mail;

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
    pub fn new(
        cpu_pool: CpuPool,
        http_client: ClientHandle,
        send_grid_conf: SendGridConf,
    ) -> Self {
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

        Box::new(self.cpu_pool.spawn_fn(move || {
            let url = format!(
                "{}/{}",
                config.api_addr.clone(),
                config.send_mail_path.clone()
            );

            let payload = from_simple_mail(mail, config.from_email.clone());
            serde_json::to_string(
                &payload
            ).into_future()
            .map_err(|e| {
                error!("Couldn't parse payload");
                ServiceError::from(e)
            })
            .and_then(move |body| {
                info!("Sending payload: {}", &body);

                let mut headers = Headers::new();
                let api_key = config.api_key.clone();
                headers.set(
                    Authorization(
                        Bearer {
                            token: api_key
                        }
                    )
                );
                headers.set(
                    ContentType(mime::APPLICATION_JSON)
                );

                http_clone
                    .request::<serde_json::Value>(Method::Post, url, Some(body), Some(headers))
                    .map_err(|e| {
                        error!("Couldn't complete http request");
                        ServiceError::from(e)
                    })
                    .map(|_| "Ok".to_string())
            })
        }))
    }
}
