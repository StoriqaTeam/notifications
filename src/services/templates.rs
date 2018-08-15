use failure::Error as FailureError;
use failure::Fail;
use futures::prelude::*;
use futures_cpupool::CpuPool;

use stq_http::client::ClientHandle;
use stq_types::UserId;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use r2d2::{ManageConnection, Pool};

use super::types::ServiceFuture;
use errors::Error;
use models::{Template, UpdateTemplate};
use repos::{ReposFactory, TemplateVariant};

pub trait TemplatesService {
    /// Get template by name
    fn get_template_by_name(self, template_name: TemplateVariant) -> ServiceFuture<String>;
    // Update template by name
    fn update_template(self, template_name: TemplateVariant, payload: UpdateTemplate) -> ServiceFuture<Template>;
}

pub struct TemplatesServiceImpl<T, M, F>
where
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
{
    pub cpu_pool: CpuPool,
    pub http_client: ClientHandle,
    pub user_id: Option<UserId>,
    pub db_pool: Pool<M>,
    pub repo_factory: F,
}

impl<T, M, F> TemplatesServiceImpl<T, M, F>
where
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
{
    pub fn new(cpu_pool: CpuPool, http_client: ClientHandle, user_id: Option<UserId>, db_pool: Pool<M>, repo_factory: F) -> Self {
        Self {
            cpu_pool,
            http_client,
            user_id,
            db_pool,
            repo_factory,
        }
    }
}

impl<T, M, F> TemplatesService for TemplatesServiceImpl<T, M, F>
where
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
{
    fn get_template_by_name(self, template_name: TemplateVariant) -> ServiceFuture<String> {
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();
        let user_id = self.user_id;

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let templates_repo = repo_factory.create_templates_repo(&*conn, user_id);
                            templates_repo
                                .get_template_by_name(template_name.to_string())
                                .map(|template| template.data)
                                .map_err(|e| e.context(format!("Get template by name {} error occured", template_name)).into())
                        })
                })
                .map_err(|e: FailureError| {
                    e.context("Service MailService, get_template_by_name endpoint error occured.")
                        .into()
                }),
        )
    }

    fn update_template(self, template_name: TemplateVariant, payload: UpdateTemplate) -> ServiceFuture<Template> {
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();
        let user_id = self.user_id;

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let templates_repo = repo_factory.create_templates_repo(&*conn, user_id);
                            templates_repo
                                .update(template_name.to_string(), payload)
                                .map_err(|e| e.context(format!("Update template {} error occured", template_name)).into())
                        })
                })
                .map_err(|e: FailureError| e.context("Service MailService, update_template endpoint error occured.").into()),
        )
    }
}
