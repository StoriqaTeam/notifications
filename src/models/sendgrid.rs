use super::mail::SimpleMail;

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

pub fn from_simple_mail(simple_mail: SimpleMail, from_email: String) -> SendGridPayload {
    let mut to: Vec<Address> = Vec::new();
    to.push(Address {
        email: simple_mail.to.clone(),
        name: None,
    });

    let mut personalizations: Vec<Personalization> = Vec::new();
    personalizations.push(Personalization { to });

    let from = Address {
        email: from_email.clone(),
        name: None,
    };

    let subject = simple_mail.subject.clone();

    let mut content: Vec<Content> = Vec::new();
    content.push(Content {
        type_field: "text/plain".to_string(),
        value: simple_mail.text.clone(),
    });

    let payload = SendGridPayload {
        personalizations,
        from,
        subject,
        content,
    };

    payload
}
