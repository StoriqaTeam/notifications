use futures::future;
use futures_cpupool::CpuPool;

use models::SimpleMail;

use super::types::ServiceFuture;
use super::error::ServiceError;

use lettre;
use lettre::{SimpleSendableEmail, EmailTransport, EmailAddress, SmtpTransport};
use lettre::smtp::SmtpTransportBuilder;
use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::smtp::client::net::{ClientTlsParameters, DEFAULT_TLS_PROTOCOLS};
use lettre_email::EmailBuilder;

use native_tls::TlsConnector;

pub trait MailService {
    /// Send simple mail
    fn send_simple_mail(&self, mail: SimpleMail) -> ServiceFuture<String>;
}

/// Mail service, responsible for sending emails
pub struct MailServiceImpl {
    pub cpu_pool: CpuPool,
}

impl MailServiceImpl {
    pub fn new(cpu_pool: CpuPool) -> Self {
        Self {
            cpu_pool,
        }
    }
}

impl MailService for MailServiceImpl {

    fn send_simple_mail(&self, mail: SimpleMail) -> ServiceFuture<String> {
        println!("{:?}", mail);

        let email = EmailBuilder::new()
            .to(("user@localhost", "Tema"))
            .from("user@example.com")
            .subject(mail.subject.clone())
            .text(mail.text.clone())
            .build()
            .unwrap();

        let mut mailer =
            SmtpTransport::builder_unencrypted_localhost().unwrap().build();

        let result = mailer.send(&email);

        if result.is_ok() {
            println!("Email sent");
            Box::new(future::ok("Ok".to_string()))
        } else {
            println!("Could not send email: {:?}", result);
            Box::new(future::ok("Error".to_string()))
        }
    }
}