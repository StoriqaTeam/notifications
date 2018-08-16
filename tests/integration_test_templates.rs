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

use lib::models::*;
use lib::repos::TemplateVariant;

use stq_http::client::ClientHandle as HttpClientHandle;
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
}

#[test]
fn test_get_template() {
    let base_url = common::setup();

    let templates = vec![
        "users/template-order-update-state".to_owned(),
        "stores/template-order-update-state".to_owned(),
        "users/template-order-create".to_owned(),
        "stores/template-order-create".to_owned(),
        "users/template-email-verification".to_owned(),
        "users/template-apply-email-verification".to_owned(),
        "users/template-password-reset".to_owned(),
        "users/template-apply-password-reset".to_owned(),
    ];

    {
        let user_id = UserId(1);
        let mut rpc = RpcClient::new(base_url.clone(), user_id);
        for template in templates.iter() {
            let template_result = rpc.core.run(rpc.http_client.request_with_auth_header::<String>(
                Method::Get,
                format!("{}/{}", rpc.base_url, template),
                None,
                Some(rpc.user.to_string()),
            ));

            assert!(template_result.is_ok());
        }
    }

    {
        let user_id = UserId(123);
        let mut rpc = RpcClient::new(base_url.clone(), user_id);
        for template in templates.iter() {
            assert!(
                rpc.core
                    .run(rpc.http_client.request_with_auth_header::<String>(
                        Method::Get,
                        format!("{}/{}", rpc.base_url, template),
                        None,
                        Some(rpc.user.to_string()),
                    ))
                    .is_err()
            );
        }
    }
}
