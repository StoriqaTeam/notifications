#![allow(proc_macro_derive_resolution_fallback)]
extern crate chrono;
extern crate config as config_crate;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate futures_cpupool;
extern crate hyper;
extern crate hyper_tls;
#[macro_use]
extern crate log;
extern crate r2d2;
extern crate rand;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate handlebars;
extern crate mime;
extern crate serde_json;
extern crate tokio_core;
extern crate tokio_signal;
extern crate uuid;
#[macro_use]
extern crate sentry;
extern crate base64;
extern crate sha1;

#[macro_use]
extern crate stq_http;
extern crate stq_logging;
extern crate stq_router;
extern crate stq_static_resources;
extern crate stq_types;

pub mod config;
pub mod controller;
pub mod errors;
pub mod models;
pub mod repos;
pub mod schema;
pub mod sentry_integration;
pub mod services;

use std::process;
use std::sync::Arc;

use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use futures::future;
use futures::prelude::*;
use futures_cpupool::CpuPool;
use hyper::server::Http;
use tokio_core::reactor::Core;

use stq_http::controller::Application;

use controller::context::StaticContext;
use repos::acl::RolesCacheImpl;
use repos::repo_factory::ReposFactoryImpl;
use services::emarsys::{EmarsysClient, EmarsysClientImpl};
use services::mocks::emarsys::EmarsysClientMock;

/// Starts new web service from provided `Config`
pub fn start_server<F: FnOnce() + 'static>(config: config::Config, port: &Option<i32>, callback: F) {
    let thread_count = config.server.thread_count;
    let cpu_pool = CpuPool::new(thread_count);
    // Prepare reactor
    let mut core = Core::new().expect("Unexpected error creating event loop core");
    let handle = Arc::new(core.handle());

    // Prepare database pool
    let database_url: String = config.server.database.parse().expect("Database URL must be set in configuration");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let db_pool = r2d2::Pool::builder().build(manager).expect("Failed to create connection pool");

    // Prepare server
    let address = {
        let port = port.as_ref().unwrap_or(&config.server.port);
        format!("{}:{}", config.server.host, port).parse().expect("Could not parse address")
    };

    // Roles cache
    let roles_cache = RolesCacheImpl::default();

    // Repo factory
    let repo_factory = ReposFactoryImpl::new(roles_cache.clone());

    let client = stq_http::client::Client::new(&config.to_http_config(), &handle);
    let client_handle = client.handle();
    let client_stream = client.stream();
    handle.spawn(client_stream.for_each(|_| Ok(())));

    // TODO: Replace with mock implementation if needed.
    let emarsys_client: Arc<EmarsysClient> = if config.clone().testmode.map(|t| t.emarsys).unwrap_or(false) {
        let emarsys_client: Arc<EmarsysClient> = Arc::new(EmarsysClientMock::new());
        emarsys_client
    } else {
        let emarsys_client_mock: Arc<EmarsysClient> = Arc::new(EmarsysClientImpl {
            config: config.emarsys.clone().expect("Failed to load emarsys config"),
            client_handle: client_handle.clone(),
        });
        emarsys_client_mock
    };

    let context = StaticContext::new(
        db_pool,
        cpu_pool,
        client_handle,
        Arc::new(config),
        repo_factory,
        emarsys_client,
    );

    let serve = Http::new()
        .serve_addr_handle(&address, &*handle, move || {
            // Prepare application
            let controller = controller::ControllerImpl::new(context.clone());
            let app = Application::<errors::Error>::new(controller);

            Ok(app)
        })
        .unwrap_or_else(|reason| {
            eprintln!("Http Server Initialization Error: {}", reason);
            process::exit(1);
        });

    handle.spawn(
        serve
            .for_each({
                let handle = handle.clone();
                move |conn| {
                    handle.spawn(conn.map(|_| ()).map_err(|why| eprintln!("Server Error: {:?}", why)));
                    Ok(())
                }
            })
            .map_err(|_| ()),
    );

    info!("Listening on http://{}, threads: {}", address, thread_count);
    handle.spawn_fn(move || {
        callback();
        future::ok(())
    });

    core.run(tokio_signal::ctrl_c().flatten_stream().take(1u64).for_each(|()| {
        info!("Ctrl+C received. Exit");
        Ok(())
    }))
    .unwrap();
}
