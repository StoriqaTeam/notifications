extern crate futures;
extern crate hyper;
extern crate notifications_lib as lib;
extern crate serde_json;
extern crate stq_http;
extern crate stq_static_resources;
extern crate stq_types;
extern crate tokio_core;

pub mod common;

use hyper::Method;

use std::result;
use stq_http::client::{self, ClientHandle as HttpClientHandle};
use stq_types::*;
use tokio_core::reactor::Core;

struct RpcClient {
    http_client: HttpClientHandle,
    core: Core,
    base_url: String,
    user: UserId,
}

impl RpcClient {
    fn new(base_url: String, user: UserId) -> Self {
        let (core, http_client) = common::make_utils();
        RpcClient {
            http_client,
            core,
            base_url,
            user,
        }
    }

    fn request_template(&mut self, method: Method, request_path: String, body: Option<String>) -> result::Result<String, client::Error> {
        self.core.run(self.http_client.request_with_auth_header::<String>(
            method,
            format!("{}/{}", self.base_url, request_path),
            body,
            Some(self.user.to_string()),
        ))
    }
}

fn init_templates_paths() -> Vec<String> {
    vec![
        "users/template-order-update-state".to_owned(),
        "stores/template-order-update-state".to_owned(),
        "users/template-order-create".to_owned(),
        "stores/template-order-create".to_owned(),
        "users/template-email-verification".to_owned(),
        "users/template-apply-email-verification".to_owned(),
        "users/template-password-reset".to_owned(),
        "users/template-apply-password-reset".to_owned(),
    ]
}

#[test]
fn test_get_template() {
    let base_url = common::setup();

    let templates = init_templates_paths();

    {
        let user_id = UserId(1);
        let mut rpc = RpcClient::new(base_url.clone(), user_id);
        for template in templates.iter() {
            let template_result = rpc.request_template(Method::Get, template.clone(), None);
            assert!(template_result.is_ok());
        }
    }

    {
        let user_id = UserId(123);
        let mut rpc = RpcClient::new(base_url.clone(), user_id);
        for template in templates.iter() {
            let template_result = rpc.request_template(Method::Get, template.clone(), None);
            assert!(template_result.is_err());
        }
    }
}

#[test]
fn test_update_template() {
    let base_url = common::setup();

    let templates = init_templates_paths();
    let template_mock = "<html>{{param1}}</html>".to_owned();

    {
        let user_id = UserId(1);
        let mut rpc = RpcClient::new(base_url.clone(), user_id);
        for template in templates.iter() {
            let template_result = rpc.request_template(Method::Put, template.clone(), Some(template_mock.clone()));
            assert!(template_result.is_ok());
        }
    }

    {
        let user_id = UserId(123);
        let mut rpc = RpcClient::new(base_url.clone(), user_id);
        for template in templates.iter() {
            let template_result = rpc.request_template(Method::Put, template.clone(), Some(template_mock.clone()));
            assert!(template_result.is_err());
        }
    }
}
