use std::sync::Arc;

use failure::Fail;
use futures::future;
use futures::prelude::*;
use futures_cpupool::CpuPool;
use hyper::server::Request;
use hyper::{Get, Post};

use stq_http::client::ClientHandle;
use stq_http::controller::Controller;
use stq_http::controller::ControllerFuture;
use stq_http::request_util::parse_body;
use stq_http::request_util::serialize_future;
use stq_router::RouteParser;

use self::routes::Route;
use config;
use errors::Error;
use models;
use services::mail::{MailService, SendGridServiceImpl};
use services::system::{SystemService, SystemServiceImpl};

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
        let system_service = SystemServiceImpl::new();

        let mail_service = SendGridServiceImpl::new(self.cpu_pool.clone(), self.http_client.clone(), self.config.sendgrid.clone());

        let path = req.path().to_string();

        match (&req.method().clone(), self.route_parser.test(req.path())) {
            // GET /healthcheck
            (&Get, Some(Route::Healthcheck)) => {
                trace!("Received healthcheck request");
                serialize_future(system_service.healthcheck())
            }

            // POST /sendmail
            (&Post, Some(Route::SendMail)) => serialize_future(
                parse_body::<models::SimpleMail>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // POST /sendmail in SimpleMail failed!")
                            .context(Error::Parse)
                            .into()
                    })
                    .and_then(move |mail| mail_service.send_mail(mail)),
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
