//! Config module contains the top-level config for the app.

use std::env;
use config_crate::{Config as RawConfig, ConfigError, Environment, File};

/// Basic settings - HTTP binding address and database DSN
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: Server,
    pub client: Client,
    pub smtp: SmtpConf,
}

/// Common server settings
#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub address: String,
    pub thread_count: usize,
}

/// Http client settings
#[derive(Debug, Deserialize, Clone)]
pub struct Client {
    pub http_client_retries: usize,
    pub http_client_buffer_size: usize,
    pub dns_worker_thread_count: usize,
}

/// Smtp client settings
#[derive(Debug, Deserialize, Clone)]
pub struct SmtpConf {
    pub smtp_sock_addr: String,
    pub smtp_domain: String,
    pub require_tls: bool,
    pub hello_name: String,
    pub timeout_secs: u64,
    pub username: String,
    pub password: String,
}

/// Creates new app config struct
/// #Examples
/// ```
/// use users_lib::config::*;
///
/// let config = Config::new();
/// ```
impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = RawConfig::new();
        s.merge(File::with_name("config/base"))?;

        // Note that this file is _optional_
        let env = env::var("RUN_MODE").unwrap_or("development".into());
        s.merge(File::with_name(&format!("config/{}", env)).required(false))?;

        // Add in settings from the environment (with a prefix of STQ_USERS)
        s.merge(Environment::with_prefix("STQ_NOTIF"))?;

        s.try_into()
    }
}