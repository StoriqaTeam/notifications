extern crate futures;
extern crate rand;
extern crate stq_http;
extern crate tokio_core;

extern crate notifications_lib as lib;

use self::futures::prelude::*;
use self::rand::Rng;
use self::stq_http::client::{Client as HttpClient, ClientHandle as HttpClientHandle, Config as HttpConfig};
use self::tokio_core::reactor::Core;
use std::sync::mpsc::channel;
use std::thread;

pub fn setup() -> String {
    let (tx, rx) = channel::<bool>();
    let mut rng = rand::thread_rng();
    let port = rng.gen_range(50000, 60000);
    thread::spawn({
        let tx = tx.clone();
        move || {
            let config = lib::config::Config::new().expect("Can't load app config!");
            lib::start_server(config, &Some(port), move || {
                let _ = tx.send(true);
            });
        }
    });
    rx.recv().unwrap();

    format!("http://localhost:{}", port)
}

pub fn make_utils() -> (Core, HttpClientHandle) {
    let core = Core::new().expect("Unexpected error creating event loop core");
    let client = HttpClient::new(
        &HttpConfig {
            http_client_retries: 3,
            http_client_buffer_size: 3,
        },
        &core.handle(),
    );
    let client_handle = client.handle();
    core.handle().spawn(client.stream().for_each(|_| Ok(())));
    (core, client_handle)
}
