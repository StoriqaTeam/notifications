pub mod routes;

use futures::prelude::*;
use futures::future;
use hyper::Method;
use hyper::{Delete, Get, Post, Put};
use hyper::server::Request;
use std::sync::Arc;

use stq_http::client::ClientHandle as HttpClientHandle;
use stq_http::controller::{Controller};
use stq_http::errors::ControllerError;
use stq_http::request_util::{read_body, ControllerFuture};
use stq_http::request_util::serialize_future;
use stq_router::RouteParser;

use config;

use self::routes::Route;
use services::system::{SystemService, SystemServiceImpl};

pub struct ControllerImpl {
    pub config: config::Config,
    pub route_parser: Arc<RouteParser<Route>>,
    pub http_client: Arc<HttpClientHandle>,
}

impl ControllerImpl {
    /// Create a new controller based on services
    pub fn new(config: config::Config, http_client: Arc<HttpClientHandle>) -> Self {
        let route_parser = Arc::new(routes::create_route_parser());
        Self {
            config,
            route_parser,
            http_client,
        }
    }
}

impl Controller for ControllerImpl {
    fn call(&self, req: Request) -> ControllerFuture {
        let system_service = SystemServiceImpl::new();

        match (req.method(), self.route_parser.test(req.path())) {

            // GET /healthcheck
            (&Get, Some(Route::Healthcheck)) => serialize_future(system_service.healthcheck()),

            // POST send simple mail
            (&Method::Post, Some(Route::SimpleMail)) => serialize_future(
                read_body(req.body())
                    .map_err(|e| ControllerError::UnprocessableEntity(e.into()))
                    .and_then({

                        move |s| {
                            Ok("d").into_future()
                        }
                    }),
            ),
            _ => Box::new(future::err(ControllerError::NotFound)),
        }
    }
}