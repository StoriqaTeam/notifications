use failure::Error as FailureError;
use failure::Fail;
use futures::prelude::*;
use handlebars::Handlebars;
use mime::{TEXT_HTML, TEXT_PLAIN};
use serde::Serialize;

use stq_static_resources::*;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use r2d2::ManageConnection;

use super::types::ServiceFuture;
use config::SendGridConf;
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
        let SendGridConf { from_email, from_name, .. } = self.static_context.config.sendgrid.clone();

        let sendgrid_service = self.static_context.sendgrid_service.clone();
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
                    .map(move |text| {
                        let mut send_mail = mail.into_send_mail();
                        send_mail.text = text;
                        SendGridPayload::from_send_mail(send_mail, from_email.clone(), from_name.clone(), TEXT_HTML)
                    })
            })
            .map_err(|e: FailureError| e.context("Mail service, send_email_with_template endpoint error occured.").into())
            .and_then(move |payload| {
                debug!("Sending payload: {:?}", &payload);
                info!("prepare for sending email from template {:?}", template_name);
                sendgrid_service
                    .send(payload)
                    .map_err(|e| e.context("SendgridService failed").into())
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
        let SendGridConf { from_email, from_name, .. } = self.static_context.config.sendgrid.clone();
        let sendgrid_service = self.static_context.sendgrid_service.clone();

        let payload = SendGridPayload::from_send_mail(mail, from_email.clone(), from_name.clone(), TEXT_PLAIN);

        debug!("Sending payload: {:?}", payload);
        info!("prepare for sending simple email");

        Box::new(
            sendgrid_service
                .send(payload)
                .map_err(|e| e.context("SendgridService failed").into()),
        )
    }
}
