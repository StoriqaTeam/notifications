use failure::Error as FailureError;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use r2d2::ManageConnection;

use repos::ReposFactory;
use services::types::{Service, ServiceFuture};
use stq_static_resources::TemplateVariant;

pub trait TemplatesService {
    /// Get template by name
    fn get_template_by_name(self, template_name: TemplateVariant) -> ServiceFuture<String>;
    // Update template by name
    fn update_template(self, template_name: TemplateVariant, text: String) -> ServiceFuture<String>;
}

impl<T, M, F> TemplatesService for Service<T, M, F>
where
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
{
    fn get_template_by_name(self, template_name: TemplateVariant) -> ServiceFuture<String> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let templates_repo = repo_factory.create_templates_repo(&*conn, user_id);
            templates_repo
                .get_template_by_name(template_name)
                .map(|template| template.data)
                .map_err(|e: FailureError| {
                    e.context("Service MailService, get_template_by_name endpoint error occurred.")
                        .into()
                })
        })
    }

    fn update_template(self, template_name: TemplateVariant, text: String) -> ServiceFuture<String> {
        let repo_factory = self.static_context.repo_factory.clone();
        let user_id = self.dynamic_context.user_id;

        self.spawn_on_pool(move |conn| {
            let templates_repo = repo_factory.create_templates_repo(&*conn, user_id);
            templates_repo
                .update(template_name, text)
                .map(|template| template.data)
                .map_err(|e: FailureError| e.context("Service MailService, update_template endpoint error occurred.").into())
        })
    }
}
