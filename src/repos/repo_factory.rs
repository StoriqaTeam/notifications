use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;

use stq_types::*;

use models::*;
use repos::legacy_acl::{Acl, SystemACL};
use repos::*;

pub trait ReposFactory<C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static>: Clone + Send + 'static {
    fn create_user_roles_repo<'a>(&self, db_conn: &'a C) -> Box<UserRolesRepo + 'a>;
    fn create_templates_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<TemplatesRepo + 'a>;
}

#[derive(Clone)]
pub struct ReposFactoryImpl {
    roles_cache: RolesCacheImpl,
}

impl ReposFactoryImpl {
    pub fn new(roles_cache: RolesCacheImpl) -> Self {
        Self { roles_cache }
    }

    pub fn get_roles<'a, C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static>(
        &self,
        id: UserId,
        db_conn: &'a C,
    ) -> Vec<UsersRole> {
        self.create_user_roles_repo(db_conn).list_for_user(id).ok().unwrap_or_default()
    }

    fn get_acl<'a, T, C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static>(
        &self,
        db_conn: &'a C,
        user_id: Option<UserId>,
    ) -> Box<Acl<Resource, Action, Scope, FailureError, T>> {
        user_id.map_or(
            Box::new(UnauthorizedAcl::default()) as Box<Acl<Resource, Action, Scope, FailureError, T>>,
            |id| {
                let roles = self.get_roles(id, db_conn);
                (Box::new(ApplicationAcl::new(roles, id)) as Box<Acl<Resource, Action, Scope, FailureError, T>>)
            },
        )
    }
}

impl<C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ReposFactory<C> for ReposFactoryImpl {
    fn create_user_roles_repo<'a>(&self, db_conn: &'a C) -> Box<UserRolesRepo + 'a> {
        Box::new(UserRolesRepoImpl::new(
            db_conn,
            Box::new(SystemACL::default()) as Box<Acl<Resource, Action, Scope, FailureError, UserRole>>,
            self.roles_cache.clone(),
        )) as Box<UserRolesRepo>
    }

    fn create_templates_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<TemplatesRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(TemplatesRepoImpl::new(db_conn, acl)) as Box<TemplatesRepo>
    }
}

#[cfg(test)]
pub mod tests {

    use std::collections::HashMap;
    use std::error::Error;
    use std::fmt;
    use std::sync::Arc;
    use std::time::SystemTime;

    use diesel::connection::AnsiTransactionManager;
    use diesel::connection::SimpleConnection;
    use diesel::deserialize::QueryableByName;
    use diesel::pg::Pg;
    use diesel::query_builder::AsQuery;
    use diesel::query_builder::QueryFragment;
    use diesel::query_builder::QueryId;
    use diesel::sql_types::HasSqlType;
    use diesel::Connection;
    use diesel::ConnectionResult;
    use diesel::QueryResult;
    use diesel::Queryable;
    use futures_cpupool::CpuPool;
    use r2d2;
    use r2d2::ManageConnection;
    use serde_json;
    use tokio_core::reactor::Handle;

    use stq_http;
    use stq_http::client::Config as HttpConfig;
    use stq_static_resources::*;
    use stq_types::*;

    use config::Config;
    use models::*;
    use repos::*;
    use services::*;

    pub const MOCK_REPO_FACTORY: ReposFactoryMock = ReposFactoryMock {};
    pub static MOCK_USER_ID: UserId = UserId(1);

    #[derive(Default, Copy, Clone)]
    pub struct ReposFactoryMock;

    impl<C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ReposFactory<C> for ReposFactoryMock {
        fn create_user_roles_repo<'a>(&self, _db_conn: &'a C) -> Box<UserRolesRepo + 'a> {
            Box::new(UserRolesRepoMock::default()) as Box<UserRolesRepo>
        }

        fn create_templates_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<TemplatesRepo + 'a> {
            Box::new(TemplatesRepoMock::default()) as Box<TemplatesRepo>
        }
    }

    #[derive(Clone, Default)]
    pub struct UserRolesRepoMock;

    impl UserRolesRepo for UserRolesRepoMock {
        fn list_for_user(&self, user_id_value: UserId) -> RepoResult<Vec<UsersRole>> {
            Ok(match user_id_value.0 {
                1 => vec![UsersRole::Superuser],
                _ => vec![UsersRole::User],
            })
        }

        fn create(&self, payload: NewUserRole) -> RepoResult<UserRole> {
            Ok(UserRole {
                id: 123,
                user_id: payload.user_id,
                role: payload.role,
            })
        }

        fn delete(&self, payload: OldUserRole) -> RepoResult<UserRole> {
            Ok(UserRole {
                id: 123,
                user_id: payload.user_id,
                role: payload.role,
            })
        }

        fn delete_by_user_id(&self, user_id_arg: UserId) -> RepoResult<UserRole> {
            Ok(UserRole {
                id: 123,
                user_id: user_id_arg,
                role: UsersRole::User,
            })
        }
    }

    #[derive(Clone, Default)]
    pub struct TemplatesRepoMock;

    impl TemplatesRepo for TemplatesRepoMock {
        fn get_template_by_name(&self, template_name: TemplateVariant) -> RepoResult<Template> {
            Ok(Template {
                id: 1,
                name: template_name,
                data: "<html></html>".to_string(),
            })
        }

        fn update(&self, template_name: TemplateVariant, payload: String) -> RepoResult<Template> {
            Ok(Template {
                id: 1,
                name: template_name,
                data: payload,
            })
        }
    }

    #[derive(Default)]
    pub struct MockConnection {
        tr: AnsiTransactionManager,
    }

    impl Connection for MockConnection {
        type Backend = Pg;
        type TransactionManager = AnsiTransactionManager;

        fn establish(_database_url: &str) -> ConnectionResult<MockConnection> {
            Ok(MockConnection::default())
        }

        fn execute(&self, _query: &str) -> QueryResult<usize> {
            unimplemented!()
        }

        fn query_by_index<T, U>(&self, _source: T) -> QueryResult<Vec<U>>
        where
            T: AsQuery,
            T::Query: QueryFragment<Pg> + QueryId,
            Pg: HasSqlType<T::SqlType>,
            U: Queryable<T::SqlType, Pg>,
        {
            unimplemented!()
        }

        fn query_by_name<T, U>(&self, _source: &T) -> QueryResult<Vec<U>>
        where
            T: QueryFragment<Pg> + QueryId,
            U: QueryableByName<Pg>,
        {
            unimplemented!()
        }

        fn execute_returning_count<T>(&self, _source: &T) -> QueryResult<usize>
        where
            T: QueryFragment<Pg> + QueryId,
        {
            unimplemented!()
        }

        fn transaction_manager(&self) -> &Self::TransactionManager {
            &self.tr
        }
    }

    impl SimpleConnection for MockConnection {
        fn batch_execute(&self, _query: &str) -> QueryResult<()> {
            Ok(())
        }
    }

    #[derive(Default)]
    pub struct MockConnectionManager;

    impl ManageConnection for MockConnectionManager {
        type Connection = MockConnection;
        type Error = MockError;

        fn connect(&self) -> Result<MockConnection, MockError> {
            Ok(MockConnection::default())
        }

        fn is_valid(&self, _conn: &mut MockConnection) -> Result<(), MockError> {
            Ok(())
        }

        fn has_broken(&self, _conn: &mut MockConnection) -> bool {
            false
        }
    }

    #[derive(Debug)]
    pub struct MockError {}

    impl fmt::Display for MockError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "SuperError is here!")
        }
    }

    impl Error for MockError {
        fn description(&self) -> &str {
            "I'm the superhero of errors"
        }

        fn cause(&self) -> Option<&Error> {
            None
        }
    }
}
