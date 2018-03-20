#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleMail {
    pub to: String,
    pub subject: String,
    pub text: String,
}