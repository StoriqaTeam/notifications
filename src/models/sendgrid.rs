use mime::Mime;
use stq_static_resources::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendGridPayload {
    pub personalizations: Vec<Personalization>,
    pub from: Address,
    pub subject: String,
    pub content: Vec<Content>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Personalization {
    pub to: Vec<Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub email: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    #[serde(rename = "type")]
    pub type_field: String,
    pub value: String,
}

impl SendGridPayload {
    pub fn from_send_mail(send_mail: SimpleMail, from_email: String, type_field: Mime) -> Self {
        let mut to: Vec<Address> = Vec::new();
        to.push(Address {
            email: send_mail.to.clone(),
            name: None,
        });

        let mut personalizations: Vec<Personalization> = Vec::new();
        personalizations.push(Personalization { to });

        let from = Address {
            email: from_email,
            name: None,
        };

        let subject = send_mail.subject.clone();

        let mut content: Vec<Content> = Vec::new();
        content.push(Content {
            type_field: type_field.to_string(),
            value: send_mail.text,
        });

        Self {
            personalizations,
            from,
            subject,
            content,
        }
    }
}
