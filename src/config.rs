//! Config module contains the top-level config for the app.
use std::env;

use stq_http;
use stq_logging::GrayLogConfig;

use sentry_integration::SentryConfig;

use config_crate::{Config as RawConfig, ConfigError, Environment, File};

/// Basic settings - HTTP binding address and database DSN
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: Server,
    pub client: Client,
    pub sendgrid: SendGridConf,
    pub graylog: Option<GrayLogConfig>,
    pub sentry: Option<SentryConfig>,
    pub emarsys: Option<EmarsysConf>,
    pub testmode: Option<TestmodeConf>
}

/// Common server settings
#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub host: String,
    pub port: i32,
    pub database: String,
    pub thread_count: usize,
}

/// Http client settings
#[derive(Debug, Deserialize, Clone)]
pub struct Client {
    pub http_client_retries: usize,
    pub http_client_buffer_size: usize,
    pub dns_worker_thread_count: usize,
    pub http_timeout_ms: u64,
}

/// Smtp client settings
#[derive(Debug, Deserialize, Clone)]
pub struct SendGridConf {
    pub api_addr: String,
    pub api_key: String,
    pub send_mail_path: String,
    pub from_email: String,
    pub from_name: String,
}

/// Emarsys api settings
#[derive(Debug, Deserialize, Clone)]
pub struct EmarsysConf {
    pub api_addr: String,
    pub username_token: String,
    pub api_secret_key: String,
    pub registration_contact_list_id: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TestmodeConf {
    pub emarsys: bool
}

/// Creates new app config struct
/// #Examples
/// ```
/// use notifications_lib::config::*;
///
/// let config = Config::new();
/// ```
impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = RawConfig::new();
        s.merge(File::with_name("config/base"))?;

        // Note that this file is _optional_
        let env = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        s.merge(File::with_name(&format!("config/{}", env)).required(false))?;

        // Add in settings from the environment (with a prefix of STQ_USERS)
        s.merge(Environment::with_prefix("STQ_NOTIF"))?;

        s.try_into()
    }

    pub fn to_http_config(&self) -> stq_http::client::Config {
        stq_http::client::Config {
            http_client_buffer_size: self.client.http_client_buffer_size,
            http_client_retries: self.client.http_client_retries,
            timeout_duration_ms: self.client.http_timeout_ms,
        }
    }
}
