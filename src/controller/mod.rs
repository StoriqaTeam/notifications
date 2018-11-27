pub mod context;
pub mod routes;

use std::str::FromStr;

use diesel::{connection::AnsiTransactionManager, pg::Pg, Connection};
use failure::Fail;
use futures::future;
use futures::prelude::*;
use hyper::{header::Authorization, server::Request, Delete, Get, Post, Put};
use r2d2::ManageConnection;

use stq_http::{
    controller::{Controller, ControllerFuture},
    errors::ErrorMessageWrapper,
    request_util::{self, parse_body, read_body, serialize_future},
};
use stq_static_resources::*;
use stq_types::*;

use self::context::{DynamicContext, StaticContext};
use self::routes::Route;
use errors::Error;
use models;
use repos::repo_factory::*;
use sentry_integration::log_and_capture_error;
use services::emarsys::EmarsysService;
use services::mail::{MailService, SimpleMailService};
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
        let user_id = get_user_id(&req);
        let correlation_token = request_util::get_correlation_token(&req);
        let dynamic_context = DynamicContext::new(user_id, correlation_token);
        let service = Service::new(self.static_context.clone(), dynamic_context);

        let path = req.path().to_string();

        let fut = match (&req.method().clone(), self.static_context.route_parser.test(req.path())) {
            (&Post, Some(Route::EmarsysCreateContact)) => serialize_future(
                parse_body::<models::CreateContactPayload>(req.body())
                    .map_err(|e| e.context("Parsing body failed, target: CreateContactPayload").context(Error::Parse).into())
                    .and_then(move |payload| service.emarsys_create_contact(payload)),
            ),
            // POST /simple-mail
            (&Post, Some(Route::SimpleMail)) => serialize_future(
                parse_body::<SimpleMail>(req.body())
                    .map_err(|e| e.context("Parsing body failed, target: SimpleMail").context(Error::Parse).into())
                    .and_then(move |mail| service.send_mail(mail)),
            ),
            // POST /users/order-update-state
            (&Post, Some(Route::OrderUpdateStateForUser)) => serialize_future(
                parse_body::<OrderUpdateStateForUser>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: OrderUpdateStateForUser")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |mail| service.send_email_with_template(TemplateVariant::OrderUpdateStateForUser, mail)),
            ),
            // GET /templates/<template_name>
            (&Get, Some(Route::Templates { template })) => serialize_future(service.get_template_by_name(template)),
            // PUT /templates/<template_name>
            (&Put, Some(Route::Templates { template })) => serialize_future(
                read_body(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: UpdateTemplate")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |text| service.update_template(template, text)),
            ),
            // POST /stores/order-update-state
            (&Post, Some(Route::OrderUpdateStateForStore)) => serialize_future(
                parse_body::<OrderUpdateStateForStore>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: OrderUpdateStateForStore")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |mail| service.send_email_with_template(TemplateVariant::OrderUpdateStateForStore, mail)),
            ),
            // POST /users/email-verification
            (&Post, Some(Route::EmailVerificationForUser)) => {
                let project = parse_query!(
                    req.query().unwrap_or_default(),
                    "project" => Project
                );
                let variant = match project {
                    Some(Project::Wallet) => TemplateVariant::WalletEmailVerificationForUser,
                    _ => TemplateVariant::EmailVerificationForUser,
                };

                serialize_future(
                    parse_body::<EmailVerificationForUser>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body failed, target: EmailVerificationForUser")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |mail| service.send_email_with_template(variant, mail)),
                )
            },
            // POST /stores/order-create
            (&Post, Some(Route::OrderCreateForStore)) => serialize_future(
                parse_body::<OrderCreateForStore>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: OrderCreateForStore")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |mail| service.send_email_with_template(TemplateVariant::OrderCreateForStore, mail)),
            ),
            // POST /users/order-create
            (&Post, Some(Route::OrderCreateForUser)) => serialize_future(
                parse_body::<OrderCreateForUser>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: OrderCreateForUser")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |mail| service.send_email_with_template(TemplateVariant::OrderCreateForUser, mail)),
            ),
            // POST /users/apply-email-verification
            (&Post, Some(Route::ApplyEmailVerificationForUser)) => {
                let project = parse_query!(
                    req.query().unwrap_or_default(),
                    "project" => Project
                );
                let variant = match project {
                    Some(Project::Wallet) => TemplateVariant::WalletApplyEmailVerificationForUser,
                    _ => TemplateVariant::ApplyEmailVerificationForUser,
                };

                serialize_future(
                    parse_body::<ApplyEmailVerificationForUser>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body failed, target: ApplyEmailVerificationForUser")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |mail| service.send_email_with_template(variant, mail)),
                )
            }
            // POST /users/password-reset
            (&Post, Some(Route::PasswordResetForUser)) => {
                let project = parse_query!(
                    req.query().unwrap_or_default(),
                    "project" => Project
                );
                let variant = match project {
                    Some(Project::Wallet) => TemplateVariant::WalletPasswordResetForUser,
                    _ => TemplateVariant::PasswordResetForUser,
                };


                serialize_future(
                    parse_body::<PasswordResetForUser>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body failed, target: PasswordResetForUser")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |mail| service.send_email_with_template(variant, mail)),
                )
            }
            ,
            // POST /users/stores/update-moderation-status
            (&Post, Some(Route::StoreModerationStatusForUser)) => serialize_future(
                parse_body::<StoreModerationStatusForUser>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: StoreModerationStatusForUser")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |mail| service.send_email_with_template(TemplateVariant::StoreModerationStatusForUser, mail)),
            ),
            // POST /users/base_products/update-moderation-status
            (&Post, Some(Route::BaseProductModerationStatusForUser)) => serialize_future(
                parse_body::<BaseProductModerationStatusForUser>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: BaseProductModerationStatusForUser")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |mail| service.send_email_with_template(TemplateVariant::BaseProductModerationStatusForUser, mail)),
            ),
            // POST /moderators/stores/update-moderation-status
            (&Post, Some(Route::StoreModerationStatusForModerator)) => serialize_future(
                parse_body::<StoreModerationStatusForModerator>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: StoreModerationStatusForModerator")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |mail| service.send_email_with_template(TemplateVariant::StoreModerationStatusForModerator, mail)),
            ),
            // POST /moderators/base_products/update-moderation-status
            (&Post, Some(Route::BaseProductModerationStatusForModerator)) => serialize_future(
                parse_body::<BaseProductModerationStatusForModerator>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body failed, target: BaseProductModerationStatusForModerator")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |mail| service.send_email_with_template(TemplateVariant::BaseProductModerationStatusForModerator, mail)),
            ),
            (&Post, Some(Route::ApplyPasswordResetForUser)) => {
                let project = parse_query!(
                    req.query().unwrap_or_default(),
                    "project" => Project
                );
                let variant = match project {
                    Some(Project::Wallet) => TemplateVariant::WalletApplyPasswordResetForUser,
                    _ => TemplateVariant::ApplyPasswordResetForUser,
                };


                serialize_future(
                    parse_body::<ApplyPasswordResetForUser>(req.body())
                        .map_err(|e| {
                            e.context("Parsing body failed, target: ApplyPasswordResetForUser")
                                .context(Error::Parse)
                                .into()
                        }).and_then(move |mail| service.send_email_with_template(variant, mail)),
                )
            }
            ,
            (Get, Some(Route::RolesByUserId { user_id })) => serialize_future({ service.get_roles(user_id) }),
            (Post, Some(Route::Roles)) => {
                serialize_future({ parse_body::<models::NewUserRole>(req.body()).and_then(move |data| service.create_user_role(data)) })
            }
            (Delete, Some(Route::Roles)) => {
                serialize_future({ parse_body::<models::RemoveUserRole>(req.body()).and_then(move |data| service.delete_user_role(data)) })
            }
            (Delete, Some(Route::RolesByUserId { user_id })) => serialize_future({ service.delete_user_role_by_user_id(user_id) }),
            (Delete, Some(Route::RoleById { id })) => serialize_future({ service.delete_user_role_by_id(id) }),

            // Fallback
            (m, _) => Box::new(future::err(
                format_err!("Request to non existing endpoint in notifications microservice! {:?} {:?}", m, path)
                    .context(Error::NotFound)
                    .into(),
            )),
        }.map_err(|err| {
            let wrapper = ErrorMessageWrapper::<Error>::from(&err);
            if wrapper.inner.code == 500 {
                log_and_capture_error(&err);
            }
            err
        });

        Box::new(fut)
    }
}

fn get_user_id(req: &Request) -> Option<UserId> {
    req.headers()
        .get::<Authorization<String>>()
        .map(|auth| auth.0.clone())
        .and_then(|id| i32::from_str(&id).ok())
        .map(UserId)
}
