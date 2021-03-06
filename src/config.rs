//! Config module contains the top-level config for the app.
use std::collections::HashMap;
use std::env;

use stq_http;
use stq_logging::GrayLogConfig;

use sentry_integration::SentryConfig;
use serde::de::{Deserializer, Visitor};
use serde::Deserialize;

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
    pub testmode: Option<TestmodeConf>,
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

/// Testmode settings
pub type TestmodeConf = HashMap<String, ApiMode>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ApiMode {
    Real,
    Mock,
}

const API_MODE_REAL: &'static str = "real";
const API_MODE_MOCK: &'static str = "mock";

const FIELDS: &'static [&'static str] = &[API_MODE_REAL, API_MODE_MOCK];

impl<'de> Deserialize<'de> for ApiMode {
    fn deserialize<D>(deserializer: D) -> Result<ApiMode, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_enum("ApiMode", FIELDS, ApiModeVisitor)
    }
}
struct ApiModeVisitor;

impl<'de> Visitor<'de> for ApiModeVisitor {
    type Value = ApiMode;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_fmt(format_args!("`{}` or `{}`", API_MODE_REAL, API_MODE_MOCK))
    }

    fn visit_str<E>(self, value: &str) -> Result<ApiMode, E>
    where
        E: serde::de::Error,
    {
        match value {
            API_MODE_REAL => Ok(ApiMode::Real),
            API_MODE_MOCK => Ok(ApiMode::Mock),
            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
        }
    }
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
