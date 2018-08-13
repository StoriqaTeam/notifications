//! Repos is a module responsible for interacting with access control lists
//! Authorization module contains authorization logic for the repo layer app

#[macro_use]
pub mod macros;
pub mod legacy_acl;
pub mod roles_cache;

pub use self::roles_cache::RolesCacheImpl;

use std::collections::HashMap;
use std::rc::Rc;

use errors::Error;
use failure::Error as FailureError;

use stq_types::{UserId, UsersRole};

use self::legacy_acl::{Acl, CheckScope};

use models::authorization::*;

pub fn check<T>(
    acl: &Acl<Resource, Action, Scope, FailureError, T>,
    resource: Resource,
    action: Action,
    scope_checker: &CheckScope<Scope, T>,
    obj: Option<&T>,
) -> Result<(), FailureError> {
    acl.allows(resource, action, scope_checker, obj).and_then(|allowed| {
        if allowed {
            Ok(())
        } else {
            Err(format_err!("Denied request to do {:?} on {:?}", action, resource)
                .context(Error::Forbidden)
                .into())
        }
    })
}

/// ApplicationAcl contains main logic for manipulation with recources
#[derive(Clone)]
pub struct ApplicationAcl {
    acls: Rc<HashMap<UsersRole, Vec<Permission>>>,
    roles: Vec<UsersRole>,
    user_id: UserId,
}

impl ApplicationAcl {
    pub fn new(roles: Vec<UsersRole>, user_id: UserId) -> Self {
        let mut hash = ::std::collections::HashMap::new();
        hash.insert(UsersRole::Superuser, vec![permission!(Resource::Templates)]);
        hash.insert(
            UsersRole::User,
            vec![
                permission!(Resource::Templates, Action::Read),
                permission!(Resource::Templates, Action::All, Scope::Owned),
                permission!(Resource::UserRoles, Action::Read, Scope::Owned),
            ],
        );

        ApplicationAcl {
            acls: Rc::new(hash),
            roles,
            user_id,
        }
    }
}
impl<T> Acl<Resource, Action, Scope, FailureError, T> for ApplicationAcl {
    fn allows(
        &self,
        resource: Resource,
        action: Action,
        scope_checker: &CheckScope<Scope, T>,
        obj: Option<&T>,
    ) -> Result<bool, FailureError> {
        let empty: Vec<Permission> = Vec::new();
        let user_id = &self.user_id;
        let hashed_acls = self.acls.clone();
        let acls = self.roles
            .iter()
            .flat_map(|role| hashed_acls.get(role).unwrap_or(&empty))
            .filter(|permission| (permission.resource == resource) && ((permission.action == action) || (permission.action == Action::All)))
            .filter(|permission| scope_checker.is_in_scope(*user_id, &permission.scope, obj));

        if acls.count() > 0 {
            Ok(true)
        } else {
            error!("Denied request from user {} to do {} on {}.", user_id, action, resource);
            Ok(false)
        }
    }
}

/// UnauthorizedAcl contains main logic for manipulation with recources
#[derive(Clone, Default)]
pub struct UnauthorizedAcl;

impl<T> Acl<Resource, Action, Scope, FailureError, T> for UnauthorizedAcl {
    fn allows(
        &self,
        resource: Resource,
        action: Action,
        _scope_checker: &CheckScope<Scope, T>,
        _obj: Option<&T>,
    ) -> Result<bool, FailureError> {
        if action == Action::Read {
            match resource {
                Resource::Templates => Ok(true),
                _ => Ok(false),
            }
        } else {
            error!("Denied unauthorized request to do {} on {}.", action, resource);
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {}
