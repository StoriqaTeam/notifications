//! Repo for templates table. Template is an entity that connects
//! templates files.

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use failure::Error as FailureError;
use failure::Fail;

use super::types::RepoResult;
use models::authorization::*;
use models::{NewTemplate, OldTemplate, Template};
use repos::legacy_acl::*;
use stq_types::UserId;

use schema::templates::dsl::*;

/// Templates repository for handling Templates
pub trait TemplatesRepo {
    /// Get template by name
    fn get_template_by_name(&self, template: String) -> RepoResult<Template>;

    /// Create a new template
    fn create(&self, payload: NewTemplate) -> RepoResult<Template>;

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
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> TemplatesRepo for TemplatesRepoImpl<'a, T> {
    fn get_template_by_name(&self, template: String) -> RepoResult<Template> {
        debug!("get template by name {}.", template);
        let query = templates.filter(name.eq(&template));
        query
            .get_result(self.db_conn)
            .map_err(|e| e.context(format!("Get template by name {} error occured", template)).into())
    }

    fn create(&self, payload: NewTemplate) -> RepoResult<Template> {
        debug!("create new template {:?}.", payload);
        let query = diesel::insert_into(templates).values(&payload);
        query
            .get_result(self.db_conn)
            .map_err(|e| e.context(format!("create new template {:?}.", payload)).into())
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
