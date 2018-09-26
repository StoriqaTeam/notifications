pub mod context;
pub mod routes;

use std::str::FromStr;
use std::sync::Arc;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Fail;
use futures::future;
use futures::prelude::*;
use futures_cpupool::CpuPool;
use hyper::header::Authorization;
use hyper::server::Request;
use hyper::{Delete, Get, Post, Put};
use r2d2::{ManageConnection, Pool};

use stq_http::client::ClientHandle;
use stq_http::controller::Controller;
use stq_http::controller::ControllerFuture;
use stq_http::request_util::serialize_future;
use stq_http::request_util::{parse_body, read_body};
use stq_router::RouteParser;
use stq_static_resources::*;
use stq_types::*;

use self::context::{DynamicContext, StaticContext};
use self::routes::Route;
use config;
use errors::Error;
use models::*;
use repos::acl::RolesCacheImpl;
use repos::repo_factory::*;
use services::mail::MailService;
use services::templates::TemplatesService;
use services::user_roles::UserRolesService;
use services::Service;

/// Controller handles route parsing and calling `Service` layer
pub struct ControllerImpl<T, M, F>
where
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
{
    pub static_context: StaticContext<T, M, F>,
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > ControllerImpl<T, M, F>
{
    /// Create a new controller based on services
    pub fn new(static_context: StaticContext<T, M, F>) -> Self {
        Self { static_context }
    }
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > Controller for ControllerImpl<T, M, F>
{
    /// Handle a request and get future response
    fn call(&self, req: Request) -> ControllerFuture {
        let headers = req.headers().clone();
        let auth_header = headers.get::<Authorization<String>>();
        let user_id = auth_header
            .map(|auth| auth.0.clone())
            .and_then(|id| i32::from_str(&id).ok())
            .map(UserId);

        debug!("User with id = '{:?}' is requesting {}", user_id, req.path());

        let dynamic_context = DynamicContext::new(user_id);

        let service = Service::new(self.static_context.clone(), dynamic_context);

        let path = req.path().to_string();

        match (&req.method().clone(), self.static_context.route_parser.test(req.path())) {
            // POST /simple-mail
            (&Post, Some(Route::SimpleMail)) => {
                debug!("User with id = '{:?}' is requesting // POST /simple-mail", user_id);
                serialize_future(
                    parse_body::<SimpleMail>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /simple-mail in SimpleMail failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |mail| service.send_mail(mail)),
                )
            }
            // POST /users/order-update-state
            (&Post, Some(Route::OrderUpdateStateForUser)) => {
                debug!("User with id = '{:?}' is requesting // POST /users/order-update-state", user_id);
                serialize_future(
                    parse_body::<OrderUpdateStateForUser>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /users/order-update-state in OrderUpdateStateForUser failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |mail| service.order_update_user(mail)),
                )
            }
            // GET /templates/<template_name>
            (&Get, Some(Route::Templates { template })) => {
                debug!(
                    "User with id = '{:?}' is requesting // GET /templates by name: {:?}",
                    user_id, template
                );
                serialize_future(service.get_template_by_name(template))
            }
            // PUT /templates/<template_name>
            (&Put, Some(Route::Templates { template })) => {
                debug!(
                    "User with id = '{:?}' is requesting // PUT /templates by name: {:?}",
                    user_id, template
                );
                serialize_future(
                    read_body(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // PUT /templates in UpdateTemplate failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |text| service.update_template(template, text)),
                )
            }
            // POST /stores/order-update-state
            (&Post, Some(Route::OrderUpdateStateForStore)) => {
                debug!("User with id = '{:?}' is requesting // POST /stores/order-update-state", user_id);
                serialize_future(
                    parse_body::<OrderUpdateStateForStore>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /stores/order-update-state in OrderUpdateStateForStore failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |mail| service.order_update_store(mail)),
                )
            }
            // POST /users/email-verification
            (&Post, Some(Route::EmailVerificationForUser)) => {
                debug!("User with id = '{:?}' is requesting // POST /users/email-verification", user_id);
                serialize_future(
                    parse_body::<EmailVerificationForUser>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /users/email-verification in EmailVerificationForUser failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |mail| service.email_verification(mail)),
                )
            }
            // POST /stores/order-create
            (&Post, Some(Route::OrderCreateForStore)) => {
                debug!("User with id = '{:?}' is requesting // POST /stores/order-create", user_id);
                serialize_future(
                    parse_body::<OrderCreateForStore>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /stores/order-create in OrderCreateForStore failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |mail| service.order_create_store(mail)),
                )
            }
            // POST /users/order-create
            (&Post, Some(Route::OrderCreateForUser)) => {
                debug!("User with id = '{:?}' is requesting // POST /users/order-create", user_id);
                serialize_future(
                    parse_body::<OrderCreateForUser>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /users/order-create in OrderCreateForUser failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |mail| service.order_create_user(mail)),
                )
            }
            // POST /users/apply-email-verification
            (&Post, Some(Route::ApplyEmailVerificationForUser)) => {
                debug!(
                    "User with id = '{:?}' is requesting // POST /users/apply-email-verification",
                    user_id
                );
                serialize_future(
                    parse_body::<ApplyEmailVerificationForUser>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /users/apply-email-verification in ApplyEmailVerificationForUser failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |mail| service.apply_email_verification(mail)),
                )
            }
            // POST /users/password-reset
            (&Post, Some(Route::PasswordResetForUser)) => {
                debug!("User with id = '{:?}' is requesting // POST /users/password-reset", user_id);
                serialize_future(
                    parse_body::<PasswordResetForUser>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /users/password-reset in PasswordResetForUser failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |mail| service.password_reset(mail)),
                )
            }
            // POST /users/apply-password-reset
            (&Post, Some(Route::ApplyPasswordResetForUser)) => {
                debug!("User with id = '{:?}' is requesting // POST /users/apply-password-reset", user_id);
                serialize_future(
                    parse_body::<ApplyPasswordResetForUser>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /users/apply-password-reset in ApplyPasswordResetForUser failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |mail| service.apply_password_reset(mail)),
                )
            }
            // GET /user_role/<user_id>
            (&Get, Some(Route::UserRole(user_id_arg))) => {
                debug!("User with id = '{:?}' is requesting  // GET /user_role/{}", user_id, user_id_arg);
                serialize_future(service.get_roles(user_id_arg))
            }
            // POST /user_roles
            (&Post, Some(Route::UserRoles)) => {
                debug!("User with id = '{:?}' is requesting  // POST /user_roles", user_id);
                serialize_future(
                    parse_body::<NewUserRole>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /user_roles in NewUserRole failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |new_role| service.create(new_role)),
                )
            }
            // DELETE /user_roles
            (&Delete, Some(Route::UserRoles)) => {
                debug!("User with id = '{:?}' is requesting  // DELETE /user_roles", user_id);
                serialize_future(
                    parse_body::<OldUserRole>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // DELETE /user_roles/<user_role_id> in OldUserRole failed!")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |old_role| service.delete(old_role)),
                )
            }
            // POST /roles/default/<user_id>
            (&Post, Some(Route::DefaultRole(user_id_arg))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // POST /roles/default/{}",
                    user_id, user_id_arg
                );
                serialize_future(service.create_default(user_id_arg))
            }
            // DELETE /roles/default/<user_id>
            (&Delete, Some(Route::DefaultRole(user_id_arg))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // DELETE /roles/default/{}",
                    user_id, user_id_arg
                );
                serialize_future(service.delete_default(user_id_arg))
            }

            // Fallback
            (m, _) => Box::new(future::err(
                format_err!("Request to non existing endpoint in notifications microservice! {:?} {:?}", m, path)
                    .context(Error::NotFound)
                    .into(),
            )),
        }
    }
}
