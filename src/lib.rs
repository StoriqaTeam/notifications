extern crate base64;
extern crate chrono;
extern crate config as config_crate;

extern crate env_logger;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate futures_cpupool;
extern crate hyper;
extern crate hyper_tls;
extern crate jsonwebtoken;
#[macro_use]
extern crate log;
extern crate rand;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate sha3;
extern crate stq_http;
extern crate stq_router;
extern crate tokio_core;
extern crate uuid;

extern crate lettre;
extern crate lettre_email;
//extern crate mime;

extern crate native_tls;

pub mod config;
pub mod controller;
pub mod services;
pub mod models;

use stq_http::client::Client as HttpClient;
use stq_http::controller::Application;


use futures::prelude::*;
use futures::future;
use futures_cpupool::CpuPool;
use hyper::server::Http;
use std::sync::Arc;
use std::process;
use tokio_core::reactor::Core;

/// Starts new web service from provided `Config`
pub fn start_server(config: config::Config) {
    // Prepare logger
    env_logger::init().unwrap();

    let thread_count = config.server.thread_count.clone();
    let cpu_pool = CpuPool::new(thread_count);
    // Prepare reactor
    let mut core = Core::new().expect("Unexpected error creating event loop core");
    let handle = Arc::new(core.handle());

    let address = config
        .server
        .address
        .parse()
        .expect("Address must be set in configuration");

    let client = HttpClient::new(
        &stq_http::client::Config {
            http_client_retries: config.client.http_client_retries,
            http_client_buffer_size: config.client.http_client_buffer_size,
        },
        &(*handle).clone(),
    );
    let client_handle = Arc::new(client.handle());
    let client_stream = client.stream();
    handle.spawn(client_stream.for_each(|_| Ok(())));

    let serve = Http::new()
        .serve_addr_handle(&address, &*handle, {
            move || {
                // Prepare application
                let app = Application {
                    controller: Box::new(controller::ControllerImpl::new(
                        config.clone(),
                        cpu_pool.clone(),
                        client_handle.clone(),
                    )),
                };

                Ok(app)
            }
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
                    handle.spawn(
                        conn.map(|_| ())
                            .map_err(|why| eprintln!("Server Error: {:?}", why)),
                    );
                    Ok(())
                }
            })
            .map_err(|_| ()),
    );

    //info!("Listening on http://{}, threads: {}", address, thread_count);
    core.run(future::empty::<(), ()>()).unwrap();
}
