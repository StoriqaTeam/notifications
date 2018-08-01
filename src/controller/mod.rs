use std::sync::Arc;

use failure::Fail;
use futures::future;
use futures::prelude::*;
use futures_cpupool::CpuPool;
use hyper::server::Request;
use hyper::Post;

use stq_http::client::ClientHandle;
use stq_http::controller::Controller;
use stq_http::controller::ControllerFuture;
use stq_http::request_util::parse_body;
use stq_http::request_util::serialize_future;
use stq_router::RouteParser;
use stq_static_resources::*;

use self::routes::Route;
use config;
use errors::Error;
use services::mail::{MailService, SendGridServiceImpl};

pub mod routes;
pub struct ControllerImpl {
    pub config: config::Config,
    pub cpu_pool: CpuPool,
    pub route_parser: Arc<RouteParser<Route>>,
    pub http_client: ClientHandle,
}

impl ControllerImpl {
    /// Create a new controller based on services
    pub fn new(config: config::Config, cpu_pool: CpuPool, http_client: ClientHandle) -> Self {
        let route_parser = Arc::new(routes::create_route_parser());
        Self {
            config,
            cpu_pool,
            route_parser,
            http_client,
        }
    }
}

impl Controller for ControllerImpl {
    fn call(&self, req: Request) -> ControllerFuture {
        let mail_service = SendGridServiceImpl::new(self.cpu_pool.clone(), self.http_client.clone(), self.config.sendgrid.clone());

        let path = req.path().to_string();

        match (&req.method().clone(), self.route_parser.test(req.path())) {
            // POST /simple-mail
            (&Post, Some(Route::SimpleMail)) => serialize_future(
                parse_body::<SimpleMail>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // POST /simple-mail in SimpleMail failed!")
                            .context(Error::Parse)
                            .into()
                    })
                    .and_then(move |mail| mail_service.send_mail(mail)),
            ),
            // POST /users/order-update-state
            (&Post, Some(Route::OrderUpdateStateForUser)) => serialize_future(
                parse_body::<OrderUpdateStateForUser>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // POST /users/order-update-state in OrderUpdateStateForUser failed!")
                            .context(Error::Parse)
                            .into()
                    })
                    .and_then(move |mail| mail_service.order_update_user(mail)),
            ),
            // POST /stores/order-update-state
            (&Post, Some(Route::OrderUpdateStateForStore)) => serialize_future(
                parse_body::<OrderUpdateStateForStore>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // POST /stores/order-update-state in OrderUpdateStateForStore failed!")
                            .context(Error::Parse)
                            .into()
                    })
                    .and_then(move |mail| mail_service.order_update_store(mail)),
            ),
            // POST /users/email-verification
            (&Post, Some(Route::EmailVerificationForUser)) => serialize_future(
                parse_body::<EmailVerificationForUser>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // POST /users/email-verification in EmailVerificationForUser failed!")
                            .context(Error::Parse)
                            .into()
                    })
                    .and_then(move |mail| mail_service.email_verification(mail)),
            ),
            // POST /stores/order-create
            (&Post, Some(Route::OrderCreateForStore)) => serialize_future(
                parse_body::<OrderCreateForStore>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // POST /stores/order-create in OrderCreateForStore failed!")
                            .context(Error::Parse)
                            .into()
                    })
                    .and_then(move |mail| mail_service.order_create_store(mail)),
            ),
            // POST /users/order-create
            (&Post, Some(Route::OrderCreateForUser)) => serialize_future(
                parse_body::<OrderCreateForUser>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // POST /users/order-create in OrderCreateForUser failed!")
                            .context(Error::Parse)
                            .into()
                    })
                    .and_then(move |mail| mail_service.order_create_user(mail)),
            ),
            // POST /users/apply-email-verification
            (&Post, Some(Route::ApplyEmailVerificationForUser)) => serialize_future(
                parse_body::<ApplyEmailVerificationForUser>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // POST /users/apply-email-verification in ApplyEmailVerificationForUser failed!")
                            .context(Error::Parse)
                            .into()
                    })
                    .and_then(move |mail| mail_service.apply_email_verification(mail)),
            ),
            // POST /users/password-reset
            (&Post, Some(Route::PasswordResetForUser)) => serialize_future(
                parse_body::<PasswordResetForUser>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // POST /users/password-reset in PasswordResetForUser failed!")
                            .context(Error::Parse)
                            .into()
                    })
                    .and_then(move |mail| mail_service.password_reset(mail)),
            ),
            // POST /users/apply-password-reset
            (&Post, Some(Route::ApplyPasswordResetForUser)) => serialize_future(
                parse_body::<ApplyPasswordResetForUser>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // POST /users/apply-password-reset in ApplyPasswordResetForUser failed!")
                            .context(Error::Parse)
                            .into()
                    })
                    .and_then(move |mail| mail_service.apply_password_reset(mail)),
            ),

            // Fallback
            (m, _) => Box::new(future::err(
                format_err!("Request to non existing endpoint in notifications microservice! {:?} {:?}", m, path)
                    .context(Error::NotFound)
                    .into(),
            )),
        }
    }
}
