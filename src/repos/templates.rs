//! Repo for templates table. Template is an entity that connects
//! templates files.

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::LoadQuery;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use failure::Error as FailureError;
use failure::Fail;

use super::acl;
use super::types::RepoResult;
use models::authorization::*;
use models::{NewTemplate, OldTemplate, Template, UpdateTemplate};
use repos::legacy_acl::*;
use stq_types::UserId;

use std::fmt;

use schema::templates::dsl::*;

/// Templates repository for handling Templates
pub trait TemplatesRepo {
    /// Get template by name
    fn get_template_by_name(&self, template: String) -> RepoResult<Template>;

    /// Create a new template
    fn create(&self, payload: NewTemplate) -> RepoResult<Template>;

    /// Update template
    fn update(&self, temlate_name: String, payload: UpdateTemplate) -> RepoResult<Template>;

    /// Delete template
    fn delete(&self, payload: OldTemplate) -> RepoResult<Template>;
}

/// Implementation of Templates trait
pub struct TemplatesRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, Template>>,
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> TemplatesRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, Template>>) -> Self {
        Self { db_conn, acl }
    }

    fn execute_query<Ty: Send + 'static, U: LoadQuery<T, Ty> + Send + 'static>(&self, query: U) -> RepoResult<Ty> {
        query.get_result::<Ty>(self.db_conn).map_err(From::from)
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> TemplatesRepo for TemplatesRepoImpl<'a, T> {
    fn get_template_by_name(&self, template_name: String) -> RepoResult<Template> {
        debug!("get template by name {}.", template_name);
        self.execute_query(templates.filter(name.eq(template_name.clone())))
            .and_then(|template| acl::check(&*self.acl, Resource::Templates, Action::Update, self, Some(&template)).map(|_| template))
            .map_err(|e: FailureError| e.context(format!("Getting template with name {} failed.", template_name)).into())
    }

    fn create(&self, payload: NewTemplate) -> RepoResult<Template> {
        debug!("create new template {:?}.", payload);
        let query = diesel::insert_into(templates).values(&payload);
        query
            .get_result(self.db_conn)
            .map_err(|e| e.context(format!("create new template {:?}.", payload)).into())
    }

    fn update(&self, template_name: String, payload: UpdateTemplate) -> RepoResult<Template> {
        debug!("Updating template with name {} and payload {:?}.", template_name, payload);
        self.execute_query(templates.filter(name.eq(template_name.clone())))
            .and_then(|template| acl::check(&*self.acl, Resource::Templates, Action::Update, self, Some(&template)))
            .and_then(|_| {
                let filter = templates.filter(name.eq(template_name.clone()));
                let query = diesel::update(filter).set(&payload);
                query.get_result(self.db_conn).map_err(From::from)
            })
            .map_err(|e: FailureError| {
                e.context(format!(
                    "Updating template with name {} and payload {:?} failed.",
                    template_name, payload
                )).into()
            })
    }

    fn delete(&self, payload: OldTemplate) -> RepoResult<Template> {
        debug!("delete template {:?}.", payload);
        let filtered = templates.filter(name.eq(payload.name.clone()));
        let query = diesel::delete(filtered);
        query
            .get_result(self.db_conn)
            .map_err(move |e| e.context(format!("delete template {:?}.", payload)).into())
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, Template>
    for TemplatesRepoImpl<'a, T>
{
    fn is_in_scope(&self, _user_id_arg: UserId, _scope: &Scope, _obj: Option<&Template>) -> bool {
        false
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TemplateVariant {
    OrderUpdateStateForUser,
    OrderUpdateStateForStore,
    OrderCreateForUser,
    OrderCreateForStore,
    EmailVerificationForUser,
    PasswordResetForUser,
    ApplyPasswordResetForUser,
    ApplyEmailVerificationForUser,
}

impl fmt::Display for TemplateVariant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TemplateVariant::OrderUpdateStateForUser => write!(f, "user_order_update"),
            TemplateVariant::OrderUpdateStateForStore => write!(f, "store_order_update"),
            TemplateVariant::OrderCreateForUser => write!(f, "user_order_create"),
            TemplateVariant::OrderCreateForStore => write!(f, "store_order_create"),
            TemplateVariant::EmailVerificationForUser => write!(f, "user_email_verification"),
            TemplateVariant::PasswordResetForUser => write!(f, "user_reset_password"),
            TemplateVariant::ApplyPasswordResetForUser => write!(f, "user_reset_password_apply"),
            TemplateVariant::ApplyEmailVerificationForUser => write!(f, "user_email_verification_apply"),
        }
    }
}
