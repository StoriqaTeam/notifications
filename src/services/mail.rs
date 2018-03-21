use std::error::Error;
use std::time::Duration;

use futures_cpupool::CpuPool;

use models::SimpleMail;
use config::SmtpConf;
use super::types::ServiceFuture;
use super::error::ServiceError;

use lettre::EmailTransport;
use lettre::smtp::{ClientSecurity, SmtpTransportBuilder};
use lettre::smtp::authentication::Credentials;
use lettre::smtp::client::net::{ClientTlsParameters, DEFAULT_TLS_PROTOCOLS};
use lettre::smtp::extension::ClientId;
use lettre_email::EmailBuilder;

use native_tls::TlsConnector;

pub trait MailService {
    /// Send simple mail
    fn send_mail(&self, mail: SimpleMail) -> ServiceFuture<String>;
}

/// Mail service, responsible for sending emails
pub struct MailServiceImpl {
    pub cpu_pool: CpuPool,
    pub smtp_conf: SmtpConf,
}

impl MailServiceImpl {
    pub fn new(cpu_pool: CpuPool, smtp_conf: SmtpConf) -> Self {
        Self {
            cpu_pool,
            smtp_conf,
        }
    }
}

impl MailService for MailServiceImpl {
    fn send_mail(&self, mail: SimpleMail) -> ServiceFuture<String> {
        let config = self.smtp_conf.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            let email = EmailBuilder::new()
                .to(mail.to.clone())
                .from(config.username.clone())
                .subject(mail.subject.clone())
                .text(mail.text.clone())
                .build()
                .map_err(|e| ServiceError::Unknown(format!("Error constructing mail: {}", e.description())))?;

            let mut tls_builder = TlsConnector::builder().map_err(|e| {
                ServiceError::Unknown(format!(
                    "Failed to create TLS connector: {}",
                    e.description()
                ))
            })?;
            tls_builder
                .supported_protocols(DEFAULT_TLS_PROTOCOLS)
                .map_err(|e| {
                    ServiceError::Unknown(format!(
                        "Failed to set supported protocols: {}",
                        e.description()
                    ))
                })?;

            let connector = tls_builder
                .build()
                .map_err(|e| ServiceError::Unknown(format!("Failed to build connector: {}", e.description())))?;

            let tls_parameters = ClientTlsParameters::new(config.smtp_domain.clone(), connector);

            let client_security = if config.require_tls {
                ClientSecurity::Required(tls_parameters)
            } else {
                ClientSecurity::Opportunistic(tls_parameters)
            };

            let mailer = SmtpTransportBuilder::new(config.smtp_sock_addr.clone(), client_security).map_err(|e| {
                ServiceError::Unknown(format!(
                    "Unable to setup SMTP transport: {}",
                    e.description()
                ))
            })?;

            let mut mailer = mailer
                .hello_name(ClientId::Domain(config.hello_name.clone()))
                .smtp_utf8(true)
                .timeout(Some(Duration::from_secs(config.timeout_secs.clone())))
                .credentials(Credentials::new(
                    config.username.clone(),
                    config.password.clone(),
                ))
                .build();

            mailer
                .send(&email)
                .map(|_resp| "Ok".to_string())
                .map_err(|e| ServiceError::Unknown(format!("Error sending mail: {}", e.description())))
        }))
    }
}
