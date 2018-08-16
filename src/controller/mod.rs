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

use self::routes::Route;
use config;
use errors::Error;
use models::*;
use repos::repo_factory::*;
use repos::templates::TemplateVariant;
use services::mail::{MailService, SendGridServiceImpl};
use services::templates::{TemplatesService, TemplatesServiceImpl};
use services::user_roles::{UserRolesService, UserRolesServiceImpl};

/// Controller handles route parsing and calling `Service` layer
#[derive(Clone)]
pub struct ControllerImpl<T, M, F>
where
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
{
    pub db_pool: Pool<M>,
    pub config: config::Config,
    pub cpu_pool: CpuPool,
    pub route_parser: Arc<RouteParser<Route>>,
    pub repo_factory: F,
    pub http_client: ClientHandle,
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > ControllerImpl<T, M, F>
{
    /// Create a new controller based on services
    pub fn new(db_pool: Pool<M>, config: config::Config, cpu_pool: CpuPool, http_client: ClientHandle, repo_factory: F) -> Self {
        let route_parser = Arc::new(routes::create_route_parser());
        Self {
            db_pool,
            config,
            cpu_pool,
            route_parser,
            repo_factory,
            http_client,
        }
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

        let mail_service = SendGridServiceImpl::new(
            self.cpu_pool.clone(),
            self.http_client.clone(),
            user_id.clone(),
            self.config.sendgrid.clone(),
            self.db_pool.clone(),
            self.repo_factory.clone(),
        );

        let templates_service = TemplatesServiceImpl::new(
            self.cpu_pool.clone(),
            self.http_client.clone(),
            user_id.clone(),
            self.db_pool.clone(),
            self.repo_factory.clone(),
        );

        let user_roles_service = UserRolesServiceImpl::new(self.db_pool.clone(), self.cpu_pool.clone(), self.repo_factory.clone());

        let path = req.path().to_string();

        match (&req.method().clone(), self.route_parser.test(req.path())) {
            // POST /simple-mail
            (&Post, Some(Route::SimpleMail)) => {
                debug!("User with id = '{:?}' is requesting // POST /simple-mail", user_id);
                serialize_future(
                    parse_body::<SimpleMail>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // POST /simple-mail in SimpleMail failed!")
                                .context(Error::Parse)
                                .into()
                        })
                        .and_then(move |mail| mail_service.send_mail(mail)),
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
                        })
                        .and_then(move |mail| mail_service.order_update_user(mail)),
                )
            }
            // GET /users/template-order-update-state
            (&Get, Some(Route::TemplateOrderUpdateStateForUser)) => {
                debug!(
                    "User with id = '{:?}' is requesting // GET /users/template-order-update-state",
                    user_id
                );
                serialize_future(templates_service.get_template_by_name(TemplateVariant::OrderUpdateStateForUser))
            }
            // PUT /users/template-order-update-state
            (&Put, Some(Route::TemplateOrderUpdateStateForUser)) => {
                debug!(
                    "User with id = '{:?}' is requesting // PUT /users/template-order-update-state",
                    user_id
                );
                serialize_future(
                    read_body(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // PUT /users/template-order-update-state in UpdateTemplate failed!")
                                .context(Error::Parse)
                                .into()
                        })
                        .and_then(move |text| templates_service.update_template(TemplateVariant::OrderUpdateStateForUser, text)),
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
                        })
                        .and_then(move |mail| mail_service.order_update_store(mail)),
                )
            }
            // GET /stores/template-order-update-state
            (&Get, Some(Route::TemplateOrderUpdateStateForStore)) => {
                debug!(
                    "User with id = '{:?}' is requesting // GET /stores/template-order-update-state",
                    user_id
                );
                serialize_future(templates_service.get_template_by_name(TemplateVariant::OrderUpdateStateForStore))
            }
            // PUT /stores/template-order-update-state
            (&Put, Some(Route::TemplateOrderUpdateStateForStore)) => {
                debug!(
                    "User with id = '{:?}' is requesting // PUT /stores/template-order-update-state",
                    user_id
                );
                serialize_future(
                    read_body(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // PUT /stores/template-order-update-state in UpdateTemplate failed!")
                                .context(Error::Parse)
                                .into()
                        })
                        .and_then(move |text| templates_service.update_template(TemplateVariant::OrderUpdateStateForStore, text)),
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
                        })
                        .and_then(move |mail| mail_service.email_verification(mail)),
                )
            }
            // GET /users/template-email-verification
            (&Get, Some(Route::TemplateEmailVerificationForUser)) => {
                debug!(
                    "User with id = '{:?}' is requesting // GET /users/template-email-verification",
                    user_id
                );
                serialize_future(templates_service.get_template_by_name(TemplateVariant::EmailVerificationForUser))
            }
            // PUT /users/template-email-verification
            (&Put, Some(Route::TemplateEmailVerificationForUser)) => {
                debug!(
                    "User with id = '{:?}' is requesting // PUT /users/template-email-verification",
                    user_id
                );
                serialize_future(
                    read_body(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // PUT /users/template-email-verification in UpdateTemplate failed!")
                                .context(Error::Parse)
                                .into()
                        })
                        .and_then(move |text| templates_service.update_template(TemplateVariant::EmailVerificationForUser, text)),
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
                        })
                        .and_then(move |mail| mail_service.order_create_store(mail)),
                )
            }
            // GET /stores/template-order-create
            (&Get, Some(Route::TemplateOrderCreateForStore)) => {
                debug!("User with id = '{:?}' is requesting // GET /stores/template-order-create", user_id);
                serialize_future(templates_service.get_template_by_name(TemplateVariant::OrderCreateForStore))
            }
            // PUT /stores/template-order-create
            (&Put, Some(Route::TemplateOrderCreateForStore)) => {
                debug!("User with id = '{:?}' is requesting // PUT /stores/template-order-create", user_id);
                serialize_future(
                    read_body(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // PUT /stores/template-order-create in UpdateTemplate failed!")
                                .context(Error::Parse)
                                .into()
                        })
                        .and_then(move |text| templates_service.update_template(TemplateVariant::OrderCreateForStore, text)),
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
                        })
                        .and_then(move |mail| mail_service.order_create_user(mail)),
                )
            }
            // GET /users/template-order-create
            (&Get, Some(Route::TemplateOrderCreateForUser)) => {
                debug!("User with id = '{:?}' is requesting // GET /users/template-order-create", user_id);
                serialize_future(templates_service.get_template_by_name(TemplateVariant::OrderCreateForUser))
            }
            // PUT /users/template-order-create
            (&Put, Some(Route::TemplateOrderCreateForUser)) => {
                debug!("User with id = '{:?}' is requesting // PUT /users/template-order-create", user_id);
                serialize_future(
                    read_body(req.body())
                        .map_err(|e| {
                            e.context("Parsing body  // PUT /users/template-order-create in UpdateTemplate failed!")
                                .context(Error::Parse)
                                .into()
                        })
                        .and_then(move |text| templates_service.update_template(TemplateVariant::OrderCreateForUser, text)),
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
                        })
                        .and_then(move |mail| mail_service.apply_email_verification(mail)),
                )
            }
            // GET /users/template-apply-email-verification
            (&Get, Some(Route::TemplateApplyEmailVerificationForUser)) => {
                debug!(
                    "User with id = '{:?}' is requesting // GET /users/template-apply-email-verification",
                    user_id
                );
                serialize_future(templates_service.get_template_by_name(TemplateVariant::ApplyEmailVerificationForUser))
            }
            // PUT /users/template-apply-email-verification
            (&Put, Some(Route::TemplateApplyEmailVerificationForUser)) => {
                debug!(
                    "User with id = '{:?}' is requesting // PUT /users/template-apply-email-verification",
                    user_id
                );
                serialize_future(
                    read_body(req.body())
                        .map_err(|e| {
                            e.context("Parsing body  // PUT /users/template-apply-email-verification in UpdateTemplate failed!")
                                .context(Error::Parse)
                                .into()
                        })
                        .and_then(move |text| templates_service.update_template(TemplateVariant::ApplyEmailVerificationForUser, text)),
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
                        })
                        .and_then(move |mail| mail_service.password_reset(mail)),
                )
            }
            // GET /users/template-password-reset
            (&Get, Some(Route::TemplatePasswordResetForUser)) => {
                debug!("User with id = '{:?}' is requesting // GET /users/template-password-reset", user_id);
                serialize_future(templates_service.get_template_by_name(TemplateVariant::PasswordResetForUser))
            }
            // PUT /users/template-password-reset
            (&Put, Some(Route::TemplatePasswordResetForUser)) => {
                debug!("User with id = '{:?}' is requesting // PUT /users/template-password-reset", user_id);
                serialize_future(
                    read_body(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // PUT /users/template-password-reset in UpdateTemplate failed!")
                                .context(Error::Parse)
                                .into()
                        })
                        .and_then(move |text| templates_service.update_template(TemplateVariant::PasswordResetForUser, text)),
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
                        })
                        .and_then(move |mail| mail_service.apply_password_reset(mail)),
                )
            }
            // GET /users/template-apply-password-reset
            (&Get, Some(Route::TemplateApplyPasswordResetForUser)) => {
                debug!(
                    "User with id = '{:?}' is requesting // GET /users/template-apply-password-reset",
                    user_id
                );
                serialize_future(templates_service.get_template_by_name(TemplateVariant::ApplyPasswordResetForUser))
            }
            // PUT /users/template-apply-password-reset
            (&Put, Some(Route::TemplateApplyPasswordResetForUser)) => {
                debug!(
                    "User with id = '{:?}' is requesting // PUT /users/template-apply-password-reset",
                    user_id
                );
                serialize_future(
                    read_body(req.body())
                        .map_err(|e| {
                            e.context("Parsing body // PUT /users/template-apply-password-reset in UpdateTemplate failed!")
                                .context(Error::Parse)
                                .into()
                        })
                        .and_then(move |text| templates_service.update_template(TemplateVariant::ApplyPasswordResetForUser, text)),
                )
            }
            // GET /user_role/<user_id>
            (&Get, Some(Route::UserRole(user_id_arg))) => {
                debug!("User with id = '{:?}' is requesting  // GET /user_role/{}", user_id, user_id_arg);
                serialize_future(user_roles_service.get_roles(user_id_arg))
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
                        })
                        .and_then(move |new_role| user_roles_service.create(new_role)),
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
                        })
                        .and_then(move |old_role| user_roles_service.delete(old_role)),
                )
            }
            // POST /roles/default/<user_id>
            (&Post, Some(Route::DefaultRole(user_id_arg))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // POST /roles/default/{}",
                    user_id, user_id_arg
                );
                serialize_future(user_roles_service.create_default(user_id_arg))
            }
            // DELETE /roles/default/<user_id>
            (&Delete, Some(Route::DefaultRole(user_id_arg))) => {
                debug!(
                    "User with id = '{:?}' is requesting  // DELETE /roles/default/{}",
                    user_id, user_id_arg
                );
                serialize_future(user_roles_service.delete_default(user_id_arg))
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
