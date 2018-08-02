use std::fs::File;
use std::io::prelude::*;

use failure::Error as FailureError;
use failure::Fail;
use futures::prelude::*;
use futures_cpupool::CpuPool;
use handlebars::Handlebars;
use hyper::header::{Authorization, Bearer, ContentType};
use hyper::{mime, Headers, Method};
use mime::{TEXT_HTML, TEXT_PLAIN};
use serde::Serialize;
use serde_json;

use stq_http::client::ClientHandle;
use stq_static_resources::*;

use super::types::ServiceFuture;
use config::SendGridConf;
use errors::Error;
use models::SendGridPayload;

pub trait MailService {
    /// Send simple mail
    fn send_mail(self, mail: SimpleMail) -> ServiceFuture<()>;
    /// Send Order Update State For Store
    fn order_update_user(self, mail: OrderUpdateStateForUser) -> ServiceFuture<()>;
    /// Send Order Update State For Store
    fn order_update_store(self, mail: OrderUpdateStateForStore) -> ServiceFuture<()>;
    /// Send Order Create State For Store
    fn order_create_user(self, mail: OrderCreateForUser) -> ServiceFuture<()>;
    /// Send Order Create State For Store
    fn order_create_store(self, mail: OrderCreateForStore) -> ServiceFuture<()>;
    /// Send Email Verification For User
    fn email_verification(self, mail: EmailVerificationForUser) -> ServiceFuture<()>;
    /// Send Apply Email Verification For User
    fn apply_email_verification(self, mail: ApplyEmailVerificationForUser) -> ServiceFuture<()>;
    /// Send Password Reset For User
    fn password_reset(self, mail: PasswordResetForUser) -> ServiceFuture<()>;
    /// Send Apply Password Reset For User
    fn apply_password_reset(self, mail: ApplyPasswordResetForUser) -> ServiceFuture<()>;
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

    pub fn send_email_with_template<T>(self, template: &str, mail: T) -> Box<Future<Item = (), Error = FailureError> + Send>
    where
        T: Email + Serialize + Clone + 'static + Send,
    {
        let config = self.send_grid_conf.clone();
        let http_clone = self.http_client.clone();
        let api_key = config.api_key.clone();
        let url = format!("{}/{}", config.api_addr.clone(), config.send_mail_path.clone());
        let handlebars = Handlebars::new();
        let path = format!("{}/templates/{}", env!("OUT_DIR"), template);
        Box::new(
            File::open(path.clone())
                .map_err({
                    let path = path.clone();
                    move |e| e.context(format!("Couldn't find template file {}", path)).into()
                })
                .and_then(|mut file| {
                    let mut template = String::new();
                    file.read_to_string(&mut template)
                        .map_err(move |e| e.context(format!("Couldn't read template file {}", path)).into())
                        .map(|_| template)
                })
                .into_future()
                .and_then({
                    let mail = mail.clone();
                    move |template| {
                        handlebars
                            .render_template(&template, &mail)
                            .into_future()
                            .map_err(move |e| e.context(format!("Couldn't render template {}", template)).into())
                    }
                })
                .and_then(move |text| {
                    let mut send_mail = mail.into_send_mail();
                    send_mail.text = text;
                    let payload = SendGridPayload::from_send_mail(send_mail, config.from_email.clone(), TEXT_HTML);
                    serde_json::to_string(&payload)
                        .into_future()
                        .map_err(|e| e.context("Couldn't parse payload").into())
                })
                .and_then(move |body| {
                    debug!("Sending payload: {}", &body);
                    let mut headers = Headers::new();
                    headers.set(Authorization(Bearer { token: api_key }));
                    headers.set(ContentType(mime::APPLICATION_JSON));

                    http_clone
                        .request::<()>(Method::Post, url, Some(body), Some(headers))
                        .map_err(|e| e.context(Error::HttpClient).into())
                }),
        )
    }
}

impl MailService for SendGridServiceImpl {
    fn send_mail(self, mail: SimpleMail) -> ServiceFuture<()> {
        let http_clone = self.http_client.clone();
        let config = self.send_grid_conf.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    let url = format!("{}/{}", config.api_addr.clone(), config.send_mail_path.clone());

                    let payload = SendGridPayload::from_send_mail(mail, config.from_email.clone(), TEXT_PLAIN);
                    serde_json::to_string(&payload)
                        .into_future()
                        .map_err(|e| e.context("Couldn't parse payload").into())
                        .and_then(move |body| {
                            debug!("Sending payload: {}", &body);

                            let mut headers = Headers::new();
                            let api_key = config.api_key.clone();
                            headers.set(Authorization(Bearer { token: api_key }));
                            headers.set(ContentType(mime::APPLICATION_JSON));

                            http_clone
                                .request::<()>(Method::Post, url, Some(body), Some(headers))
                                .map_err(|e| e.context(Error::HttpClient).into())
                        })
                })
                .map_err(|e: FailureError| e.context("Mail service, send_mail endpoint error occured.").into()),
        )
    }
    /// Send Order Update State For Store
    fn order_update_user(self, mail: OrderUpdateStateForUser) -> ServiceFuture<()> {
        let cpu_pool = self.cpu_pool.clone();
        Box::new(
            cpu_pool
                .spawn_fn(move || self.send_email_with_template("user_order_update.hbr", mail))
                .map_err(|e: FailureError| e.context("Mail service, order_update_user endpoint error occured.").into()),
        )
    }
    /// Send Order Update State For Store
    fn order_update_store(self, mail: OrderUpdateStateForStore) -> ServiceFuture<()> {
        let cpu_pool = self.cpu_pool.clone();
        Box::new(
            cpu_pool
                .spawn_fn(move || self.send_email_with_template("store_order_update.hbr", mail))
                .map_err(|e: FailureError| e.context("Mail service, order_update_store endpoint error occured.").into()),
        )
    }
    /// Send Order Create State For Store
    fn order_create_user(self, mail: OrderCreateForUser) -> ServiceFuture<()> {
        let cpu_pool = self.cpu_pool.clone();
        Box::new(
            cpu_pool
                .spawn_fn(move || self.send_email_with_template("user_order_create.hbr", mail))
                .map_err(|e: FailureError| e.context("Mail service, order_create_user endpoint error occured.").into()),
        )
    }
    /// Send Order Create State For Store
    fn order_create_store(self, mail: OrderCreateForStore) -> ServiceFuture<()> {
        let cpu_pool = self.cpu_pool.clone();
        Box::new(
            cpu_pool
                .spawn_fn(move || self.send_email_with_template("store_order_create.hbr", mail))
                .map_err(|e: FailureError| e.context("Mail service, order_create_store endpoint error occured.").into()),
        )
    }
    /// Send Email Verification For User
    fn email_verification(self, mail: EmailVerificationForUser) -> ServiceFuture<()> {
        let cpu_pool = self.cpu_pool.clone();
        Box::new(
            cpu_pool
                .spawn_fn(move || self.send_email_with_template("user_email_verification.hbr", mail))
                .map_err(|e: FailureError| e.context("Mail service, email_verification endpoint error occured.").into()),
        )
    }
    /// Send Apply Email Verification For User
    fn apply_email_verification(self, mail: ApplyEmailVerificationForUser) -> ServiceFuture<()> {
        let cpu_pool = self.cpu_pool.clone();
        Box::new(
            cpu_pool
                .spawn_fn(move || self.send_email_with_template("user_email_verification_apply.hbr", mail))
                .map_err(|e: FailureError| e.context("Mail service, apply_email_verification endpoint error occured.").into()),
        )
    }
    /// Send Password Reset For User
    fn password_reset(self, mail: PasswordResetForUser) -> ServiceFuture<()> {
        let cpu_pool = self.cpu_pool.clone();
        Box::new(
            cpu_pool
                .spawn_fn(move || self.send_email_with_template("user_reset_password.hbr", mail))
                .map_err(|e: FailureError| e.context("Mail service, password_reset endpoint error occured.").into()),
        )
    }
    /// Send Apply Password Reset For User
    fn apply_password_reset(self, mail: ApplyPasswordResetForUser) -> ServiceFuture<()> {
        let cpu_pool = self.cpu_pool.clone();
        Box::new(
            cpu_pool
                .spawn_fn(move || self.send_email_with_template("user_reset_password_apply.hbr", mail))
                .map_err(|e: FailureError| e.context("Mail service, apply_password_reset endpoint error occured.").into()),
        )
    }
}
