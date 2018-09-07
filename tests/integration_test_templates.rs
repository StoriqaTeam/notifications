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

use tokio_core::reactor::Core;

use stq_http::client::{self, ClientHandle as HttpClientHandle};
use stq_static_resources::TemplateVariant;
use stq_types::*;

struct RpcClient {
    http_client: HttpClientHandle,
    core: Core,
    base_url: String,
    user: Option<UserId>,
}

impl RpcClient {
    fn new(base_url: String, user: Option<UserId>) -> Self {
        let (core, http_client) = common::make_utils();
        RpcClient {
            http_client,
            core,
            base_url,
            user,
        }
    }

    fn request_template(&mut self, method: Method, request_path: String, body: Option<String>) -> result::Result<String, client::Error> {
        let user = self.user.map_or(None, |u| Some(u.to_string()));
        self.core.run(self.http_client.request_with_auth_header::<String>(
            method,
            format!("{}/{}", self.base_url, request_path),
            body,
            user,
        ))
    }
}

fn init_templates() -> Vec<TemplateVariant> {
    vec![
        TemplateVariant::OrderCreateForUser,
        TemplateVariant::OrderUpdateStateForUser,
        TemplateVariant::OrderCreateForStore,
        TemplateVariant::OrderUpdateStateForStore,
        TemplateVariant::EmailVerificationForUser,
        TemplateVariant::PasswordResetForUser,
        TemplateVariant::ApplyPasswordResetForUser,
        TemplateVariant::ApplyEmailVerificationForUser,
    ]
}

// test get template by superuser
#[test]
fn test_get_template_superuser() {
    let user_id = UserId(1);
    let base_url = common::setup();
    let templates = init_templates();

    let mut rpc = RpcClient::new(base_url.clone(), Some(user_id));
    for template in templates.iter() {
        let template_result = rpc.request_template(Method::Get, format!("templates/{}", template), None);
        println!("{:?}", template_result);
        assert!(template_result.is_ok());
    }
}

// test get template by regular user
#[test]
fn test_get_template_regular_user() {
    let user_id = UserId(123);
    let base_url = common::setup();
    let templates = init_templates();

    let mut rpc = RpcClient::new(base_url.clone(), Some(user_id));
    for template in templates.iter() {
        let template_result = rpc.request_template(Method::Get, format!("templates/{}", template), None);
        println!("{:?}", template_result);
        assert!(template_result.is_err());
    }
}

// test get template without authorization data
#[test]
fn test_get_template_unauthorized() {
    let base_url = common::setup();
    let templates = init_templates();

    let mut rpc = RpcClient::new(base_url.clone(), None);
    for template in templates.iter() {
        let template_result = rpc.request_template(Method::Get, format!("templates/{}", template), None);
        println!("{:?}", template_result);
        assert!(template_result.is_err());
    }
}

fn create_template_mock() -> String {
    "<html>{{param1}}</html>".to_string()
}

// test update template by superuser
#[test]
fn test_update_template_superuser() {
    let user_id = UserId(1);
    let base_url = common::setup();
    let templates = init_templates();

    let mut rpc = RpcClient::new(base_url.clone(), Some(user_id));
    for template in templates.iter() {
        let template_result = rpc.request_template(Method::Put, format!("templates/{}", template), Some(create_template_mock()));
        println!("{:?}", template_result);
        assert!(template_result.is_ok());
    }
}

// test update template by regular user
#[test]
fn test_update_template_regular_user() {
    let user_id = UserId(123);
    let base_url = common::setup();
    let templates = init_templates();

    let mut rpc = RpcClient::new(base_url.clone(), Some(user_id));
    for template in templates.iter() {
        let template_result = rpc.request_template(Method::Put, format!("templates/{}", template), Some(create_template_mock()));
        println!("{:?}", template_result);
        assert!(template_result.is_err());
    }
}

// test update template without authorization data
#[test]
fn test_update_template_unauthorized() {
    let base_url = common::setup();
    let templates = init_templates();

    let mut rpc = RpcClient::new(base_url.clone(), None);
    for template in templates.iter() {
        let template_result = rpc.request_template(Method::Put, format!("templates/{}", template), Some(create_template_mock()));
        println!("{:?}", template_result);
        assert!(template_result.is_err());
    }
}
