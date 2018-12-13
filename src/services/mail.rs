use failure::Error as FailureError;
use failure::Fail;
use futures::prelude::*;
use handlebars::Handlebars;
use hyper::header::{Authorization, Bearer, ContentType};
use hyper::{mime, Headers, Method};
use mime::{TEXT_HTML, TEXT_PLAIN};
use serde::Serialize;
use serde_json;

use stq_static_resources::*;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use r2d2::ManageConnection;

use super::types::ServiceFuture;
use config::SendGridConf;
use errors::Error;
use models::SendGridPayload;
use repos::ReposFactory;
use services::Service;

pub trait MailService<E>
where
    E: Email + Serialize + Clone + 'static + Send,
{
    /// Send email fro template
    fn send_email_with_template(self, template_name: TemplateVariant, mail: E) -> Box<Future<Item = (), Error = FailureError> + Send>;
}

pub trait SimpleMailService {
    /// Send simple mail
    fn send_mail(self, mail: SimpleMail) -> ServiceFuture<()>;
}

impl<T, M, F, E> MailService<E> for Service<T, M, F>
where
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
    E: Email + Serialize + Clone + 'static + Send,
{
    fn send_email_with_template(self, template_name: TemplateVariant, mail: E) -> Box<Future<Item = (), Error = FailureError> + Send> {
        let SendGridConf {
            api_addr,
            api_key,
            send_mail_path,
            from_email,
            from_name,
        } = self.static_context.config.sendgrid.clone();

        let http_clone = self.static_context.client_handle.clone();
        let url = format!("{}/{}", api_addr.clone(), send_mail_path.clone());
        let handlebars = Handlebars::new();

        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        Box::new(
            self.spawn_on_pool(move |conn| {
                let templates_repo = repo_factory.create_templates_repo(&*conn, user_id);
                templates_repo
                    .get_template_by_name(template_name)
                    .and_then({
                        let mail = mail.clone();
                        move |template| {
                            handlebars
                                .render_template(&template.data, &mail)
                                .map_err(move |e| e.context(format!("Couldn't render template {:?}", template.name)).into())
                        }
                    })
                    .and_then(move |text| {
                        let mut send_mail = mail.into_send_mail();
                        send_mail.text = text;
                        let payload = SendGridPayload::from_send_mail(send_mail, from_email.clone(), from_name.clone(), TEXT_HTML);
                        serde_json::to_string(&payload).map_err(|e| e.context("Couldn't parse payload").into())
                    })
            })
            .map_err(|e: FailureError| e.context("Mail service, send_email_with_template endpoint error occured.").into())
            .and_then(move |body| {
                debug!("Sending payload: {}", &body);
                info!("prepare for sending email from template {:?}", template_name);

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

impl<T, M, F> SimpleMailService for Service<T, M, F>
where
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
{
    fn send_mail(self, mail: SimpleMail) -> ServiceFuture<()> {
        let http_clone = self.static_context.client_handle.clone();
        let SendGridConf {
            api_addr,
            api_key,
            send_mail_path,
            from_email,
            from_name,
        } = self.static_context.config.sendgrid.clone();
        let url = format!("{}/{}", api_addr.clone(), send_mail_path.clone());

        let payload = SendGridPayload::from_send_mail(mail, from_email.clone(), from_name.clone(), TEXT_PLAIN);

        Box::new(
            serde_json::to_string(&payload)
                .into_future()
                .map_err(|e| e.context("Couldn't parse payload").into())
                .and_then(move |body| {
                    debug!("Sending payload: {}", &body);
                    info!("prepare for sending simple email");

                    let mut headers = Headers::new();
                    headers.set(Authorization(Bearer { token: api_key }));
                    headers.set(ContentType(mime::APPLICATION_JSON));

                    http_clone
                        .request::<()>(Method::Post, url, Some(body), Some(headers))
                        .map_err(|e| e.context(Error::HttpClient).into())
                })
                .map_err(|e: FailureError| e.context("Mail service, send_mail endpoint error occured.").into()),
        )
    }
}
