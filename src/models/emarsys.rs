use std::fmt;

use chrono::{DateTime, Utc};
use failure::Error as FailureError;
use sha1::{Digest, Sha1};
use uuid::Uuid;

use stq_http::request_util::XWSSE;
use stq_types::{EmarsysId, UserId};

pub const EMAIL_FIELD: &'static str = "3";
pub const FIRST_NAME_FIELD: &'static str = "1";
pub const LAST_NAME_FIELD: &'static str = "2";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContactPayload {
    pub user_id: UserId,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteContactPayload {
    pub user_id: UserId,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedContact {
    pub user_id: UserId,
    pub emarsys_id: EmarsysId,
}

/// delete concat
/// [https://dev.emarsys.com/v2/contacts/delete-contacts]
#[derive(Debug, Clone, Deserialize)]
pub struct DeleteContactResponse {
    #[serde(rename = "replyCode")]
    pub reply_code: Option<i64>,
    /// The summary of the response
    #[serde(rename = "replyText")]
    pub reply_text: Option<String>,
    /// Contains the number of deleted contacts as well as any errors, if applicable.
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AddToContactListRequest {
    pub key_id: String,
    pub external_ids: Vec<String>,
}

/// add concat to contact list api payload
/// [https://dev.emarsys.com/v2/contact-lists/add-contacts-to-a-contact-list]
#[derive(Debug, Clone, Deserialize)]
pub struct AddToContactListResponse {
    /// The Emarsys response code
    /// [https://dev.emarsys.com/v2/response-codes/error-codes]
    #[serde(rename = "replyCode")]
    pub reply_code: Option<i64>,
    /// The summary of the response
    #[serde(rename = "replyText")]
    pub reply_text: Option<String>,
    /// The requested data.
    pub data: Option<AddToContactListResponseData>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateContactRequest {
    pub key_id: String,
    pub contacts: Vec<serde_json::Value>,
}

/// create-contacts api payload
/// [https://dev.emarsys.com/v2/contacts/create-contacts]
#[derive(Debug, Clone, Deserialize)]
pub struct CreateContactResponse {
    /// The Emarsys response code
    /// [https://dev.emarsys.com/v2/response-codes/error-codes]
    #[serde(rename = "replyCode")]
    pub reply_code: Option<i64>,
    /// The summary of the response
    #[serde(rename = "replyText")]
    pub reply_text: Option<String>,
    /// The requested data.
    pub data: Option<CreateContactResponseData>,
}

/// The requested data.
#[derive(Debug, Clone, Deserialize)]
pub struct AddToContactListResponseData {
    /// The number of contacts successfully added to the list.
    pub inserted_contacts: Option<i32>,
    /// List of errors during adding to contact list.
    pub errors: Option<serde_json::Value>,
}

/// The requested data.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateContactResponseData {
    /// List of contact identifiers (id) of successfully created contacts.
    pub ids: Option<Vec<i32>>,
    /// List of errors during creating contacts.
    pub errors: Option<serde_json::Value>,
}

#[derive(Clone)]
pub struct ApiSecretKey(String);

#[derive(Clone)]
pub struct Nonce(Uuid);

#[derive(Debug, Clone)]
pub struct Signature {
    pub username_token: String,
    pub nonce: Nonce,
    pub timestamp: DateTime<Utc>,
    pub password_digest: PasswordDigest,
}

#[derive(Debug, Clone)]
pub struct PasswordDigest {
    pub nonce: Nonce,
    pub timestamp: DateTime<Utc>,
    pub api_secret_key: ApiSecretKey,
}

impl CreateContactResponse {
    pub fn extract_cteated_id(&self) -> Result<EmarsysId, FailureError> {
        if self.reply_code == Some(0) {
            let data = self.data.as_ref().ok_or(format_err!("data field is missing"))?;
            if let Some(ref _errors) = data.errors {
                return Err(format_err!("Response data has errors"));
            }
            let ids = data.ids.as_ref().ok_or(format_err!("ids field is missing"))?;
            if ids.len() != 1 {
                return Err(format_err!("Expected only one id"));
            }
            return ids.first().map(|id| EmarsysId(*id)).ok_or(format_err!("Expected only one id"));
        }
        Err(format_err!("Reply code is not 0"))
    }
}

impl AddToContactListResponse {
    pub fn extract_inserted_contacts(&self) -> Result<i32, FailureError> {
        if self.reply_code == Some(0) {
            let data = self.data.as_ref().ok_or(format_err!("data field is missing"))?;
            return data
                .inserted_contacts
                .ok_or(format_err!("Expected inserted_contacts to be non-null"));
        }
        Err(format_err!("Reply code is not 0"))
    }
}

impl DeleteContactResponse {
    pub fn into_result(&self) -> Result<(), FailureError> {
        if self.reply_code == Some(0) {
            return Ok(());
        }
        Err(format_err!("Reply code is not 0: {:?}", self))
    }
}

impl Signature {
    pub fn new(username_token: String, api_secret_key: String) -> Signature {
        let nonce = Nonce(Uuid::new_v4());
        let timestamp = Utc::now();
        Signature {
            username_token,
            nonce: nonce.clone(),
            timestamp,
            password_digest: PasswordDigest {
                nonce,
                timestamp,
                api_secret_key: ApiSecretKey(api_secret_key),
            },
        }
    }

    pub fn calculate(&self) -> String {
        format!(
            "UsernameToken Username=\"{}\", PasswordDigest=\"{}\", Nonce=\"{}\", Created=\"{}\"",
            self.username_token,
            self.password_digest.calculate(),
            self.nonce,
            date_time_iso8601(self.timestamp),
        )
    }
}

impl PasswordDigest {
    pub fn calculate(&self) -> String {
        let hashed_string = format!("{}{}{}", self.nonce, date_time_iso8601(self.timestamp), self.api_secret_key.0);
        let sha1_hash = Sha1::digest(hashed_string.as_bytes());
        base64::encode(&bytes_to_hex(&sha1_hash))
    }
}

impl From<CreateContactPayload> for CreateContactRequest {
    fn from(data: CreateContactPayload) -> CreateContactRequest {
        CreateContactRequest {
            key_id: EMAIL_FIELD.to_string(),
            contacts: vec![serde_json::json!({
                FIRST_NAME_FIELD: data.first_name,
                LAST_NAME_FIELD: data.last_name,
                EMAIL_FIELD: data.email
            })],
        }
    }
}

impl AddToContactListRequest {
    pub fn from_email(email: String) -> AddToContactListRequest {
        AddToContactListRequest {
            key_id: EMAIL_FIELD.to_string(),
            external_ids: vec![email],
        }
    }
}

impl Into<XWSSE> for Signature {
    fn into(self) -> XWSSE {
        XWSSE(self.calculate())
    }
}

fn date_time_iso8601(time: DateTime<Utc>) -> String {
    format!("{}", time.format("%Y-%m-%dT%H:%M:%S%z"))
}

fn bytes_to_hex(slice: &[u8]) -> String {
    const HEX_ARRAY: [char; 16] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];
    let mut hex_str = String::with_capacity(slice.len() * 2);
    for byte in slice {
        hex_str.push(HEX_ARRAY[(byte >> 4) as usize]);
        hex_str.push(HEX_ARRAY[(byte & 0x0F) as usize]);
    }
    hex_str
}

impl fmt::Debug for ApiSecretKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ApiSecretKey(***)")
    }
}

impl fmt::Debug for Nonce {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Nonce({})", bytes_to_hex(self.0.as_bytes()))
    }
}

impl fmt::Display for Nonce {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", bytes_to_hex(self.0.as_bytes()))
    }
}

#[test]
fn test_password_digest_calculation() {
    //given
    let nonce = Nonce(Uuid::from_bytes(&[96, 250, 81, 181, 131, 254, 187, 250, 198, 132, 223, 104, 53, 176, 143, 198]).unwrap());
    let timestamp = DateTime::parse_from_str("2018-11-27T11:56:04+0000", "%Y-%m-%dT%H:%M:%S%:z")
        .unwrap()
        .with_timezone(&Utc);
    let api_secret_key = ApiSecretKey("somesecret".to_string());
    let password_digest = PasswordDigest {
        nonce,
        timestamp,
        api_secret_key,
    };

    //when
    let calculated_password_digest = password_digest.calculate();
    //then
    assert_eq!(
        "NTM2MGZjNzVjNzQ5M2Q1MjUzODdmMTBhNGVhMzlhNDE4NzA2MDY2ZQ==".to_string(),
        calculated_password_digest
    );
}
