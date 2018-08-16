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

use super::acl;
use super::types::RepoResult;
use models::authorization::*;
use models::{Template, UpdateTemplate};
use repos::legacy_acl::*;
use stq_types::UserId;

use std::fmt;

use schema::templates::dsl::*;

/// Templates repository for handling Templates
pub trait TemplatesRepo {
    /// Get template by name
    fn get_template_by_name(&self, template: TemplateVariant) -> RepoResult<Template>;

    /// Update template
    fn update(&self, temlate_name: TemplateVariant, payload: UpdateTemplate) -> RepoResult<Template>;
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
    fn get_template_by_name(&self, template_name: TemplateVariant) -> RepoResult<Template> {
        debug!("get template by name {}.", template_name);
        self.execute_query(templates.filter(name.eq(template_name.clone())))
            .and_then(|template| acl::check(&*self.acl, Resource::Templates, Action::Read, self, Some(&template)).map(|_| template))
            .map_err(|e: FailureError| e.context(format!("Getting template with name {} failed.", template_name)).into())
    }

    fn update(&self, template_name: TemplateVariant, payload: UpdateTemplate) -> RepoResult<Template> {
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
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, Template>
    for TemplatesRepoImpl<'a, T>
{
    fn is_in_scope(&self, _user_id: UserId, scope: &Scope, _obj: Option<&Template>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => false,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Debug, DieselTypes)]
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
            TemplateVariant::OrderUpdateStateForUser => write!(f, "order_update_state_for_user"),
            TemplateVariant::OrderUpdateStateForStore => write!(f, "order_update_state_for_store"),
            TemplateVariant::OrderCreateForUser => write!(f, "order_create_for_user"),
            TemplateVariant::OrderCreateForStore => write!(f, "order_create_for_store"),
            TemplateVariant::EmailVerificationForUser => write!(f, "email_verification_for_user"),
            TemplateVariant::PasswordResetForUser => write!(f, "password_reset_for_user"),
            TemplateVariant::ApplyPasswordResetForUser => write!(f, "apply_password_reset_for_user"),
            TemplateVariant::ApplyEmailVerificationForUser => write!(f, "apply_email_verification_for_user"),
        }
    }
}
