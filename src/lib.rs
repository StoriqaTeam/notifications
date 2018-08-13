extern crate base64;
extern crate chrono;
extern crate config as config_crate;
#[macro_use]
extern crate diesel;
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
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rand;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate handlebars;
extern crate lettre;
extern crate lettre_email;
extern crate mime;
extern crate native_tls;
extern crate notify;
extern crate serde_json;
extern crate sha3;
extern crate tokio_core;
extern crate uuid;

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
pub mod services;

use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::process;
use std::sync::{mpsc::channel, Arc, Mutex};
use std::thread;
use std::time::Duration;

use diesel::pg::PgConnection;
use futures::future;
use futures::prelude::*;
use futures_cpupool::CpuPool;
use hyper::server::Http;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use r2d2_diesel::ConnectionManager;
use tokio_core::reactor::Core;

use stq_http::controller::Application;

use repos::acl::RolesCacheImpl;
use repos::repo_factory::ReposFactoryImpl;

/// Starts new web service from provided `Config`
pub fn start_server(config: config::Config) {
    let thread_count = config.server.thread_count;
    let cpu_pool = CpuPool::new(thread_count);
    // Prepare reactor
    let mut core = Core::new().expect("Unexpected error creating event loop core");
    let handle = Arc::new(core.handle());

    // Prepare database pool
    let database_url: String = config.server.database.parse().expect("Database URL must be set in configuration");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let r2d2_pool = r2d2::Pool::builder().build(manager).expect("Failed to create connection pool");

    // Prepare server
    let address = {
        format!("{}:{}", config.server.host, config.server.port)
            .parse()
            .expect("Could not parse address")
    };

    // Roles cache
    let roles_cache = RolesCacheImpl::default();

    // Repo factory
    let repo_factory = ReposFactoryImpl::new(roles_cache);

    let http_config = stq_http::client::Config {
        http_client_retries: config.client.http_client_retries,
        http_client_buffer_size: config.client.http_client_buffer_size,
    };
    let client = stq_http::client::Client::new(&http_config, &handle);
    let client_handle = client.handle();
    let client_stream = client.stream();
    handle.spawn(client_stream.for_each(|_| Ok(())));

    let template_dir = config
        .templates
        .clone()
        .map(|t| t.path)
        .unwrap_or_else(|| format!("{}/templates", env!("OUT_DIR")));

    let templates = Arc::new(Mutex::new(HashMap::new()));

    for entry in fs::read_dir(template_dir.clone()).unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_dir() {
            let path = entry.path();
            if let Some(file_name) = path.clone().file_name() {
                if let Some(file_name) = file_name.to_str() {
                    let res = File::open(path).and_then(|mut file| {
                        let mut template = String::new();
                        file.read_to_string(&mut template).map(|_| {
                            let mut t = templates.lock().unwrap();
                            t.insert(file_name.to_string(), template);
                        })
                    });
                    match res {
                        Ok(_) => info!("Template {} added successfully.", file_name),
                        Err(e) => error!("Template {} didn't added. Error - {}.", file_name, e),
                    }
                }
            }
        }
    }

    let (tx, rx) = channel();

    let mut watcher = watcher(tx, Duration::from_secs(10)).unwrap();

    watcher.watch(template_dir, RecursiveMode::Recursive).unwrap();

    thread::spawn({
        let templates = templates.clone();
        move || loop {
            match rx.recv() {
                Ok(event) => match event {
                    DebouncedEvent::Write(p) => {
                        if let Some(file_name) = p.clone().file_name() {
                            if let Some(file_name) = file_name.to_str() {
                                let res = File::open(p).and_then(|mut file| {
                                    let mut template = String::new();
                                    file.read_to_string(&mut template).map(|_| {
                                        let mut t = templates.lock().unwrap();
                                        t.insert(file_name.to_string(), template);
                                    })
                                });
                                match res {
                                    Ok(_) => info!("Template {} updated successfully.", file_name),
                                    Err(e) => error!("Template {} updated with error - {}.", file_name, e),
                                }
                            }
                        }
                    }
                    _ => (),
                },
                Err(e) => error!("watch templates error: {:?}", e),
            }
        }
    });
    let serve = Http::new()
        .serve_addr_handle(&address, &*handle, {
            move || {
                let controller = controller::ControllerImpl::new(
                    r2d2_pool.clone(),
                    config.clone(),
                    cpu_pool.clone(),
                    client_handle.clone(),
                    templates.clone(),
                    repo_factory.clone(),
                );

                // Prepare application
                let app = Application::<errors::Error>::new(controller);

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
                    handle.spawn(conn.map(|_| ()).map_err(|why| eprintln!("Server Error: {:?}", why)));
                    Ok(())
                }
            })
            .map_err(|_| ()),
    );

    info!("Listening on http://{}, threads: {}", address, thread_count);
    core.run(future::empty::<(), ()>()).unwrap();
}
