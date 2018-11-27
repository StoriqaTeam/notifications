use std::fmt;

use chrono::{DateTime, Utc};
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;
use failure::Fail;
use futures::{Future, IntoFuture};
use hyper::header::ContentType;
use hyper::{mime, Headers, Method};
use r2d2::ManageConnection;
use sha1::{Digest, Sha1};
use uuid::Uuid;

use stq_http::request_util::XWSSE;
use stq_types::EmarsysId;

use errors::Error;
use models::{CreateContactPayload, CreatedContact};
use repos::ReposFactory;
use services::types::{Service, ServiceFuture};

pub trait EmarsysService {
    fn emarsys_create_contact(&self, payload: CreateContactPayload) -> ServiceFuture<CreatedContact>;
}

impl<T, M, F> EmarsysService for Service<T, M, F>
where
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
{
    fn emarsys_create_contact(&self, payload: CreateContactPayload) -> ServiceFuture<CreatedContact> {
        info!("sending user {} with email {} to emarsys", payload.user_id, payload.email);
        let http_clone = self.static_context.client_handle.clone();
        let user_id = payload.user_id;
        let res = self
            .static_context
            .config
            .emarsys
            .clone()
            .ok_or(format_err!(""))
            .into_future()
            .map(|emarsys_conf| {
                let signature = Signature::new(emarsys_conf.username_token, emarsys_conf.api_secret_key);
                let request = CreateContactRequest::from(payload);
                let url = format!("{}/contact", emarsys_conf.api_addr);
                (url, signature, request)
            }).inspect(|(url, signature, request)| {
                debug!(
                    "emarsys_create_contact: url=\"{}\"; signature: {:?}; request: {:?}",
                    url, signature, request
                );
            }).and_then(|(url, signature, request)| {
                serde_json::to_string(&request)
                    .map_err(|e| e.context("Couldn't serialize payload").into())
                    .map(|request_body| (url, signature, request_body))
            }).and_then(move |(url, signature, request_body)| {
                let mut headers = Headers::new();
                headers.set(ContentType(mime::APPLICATION_JSON));
                let xwsse: XWSSE = signature.into();
                headers.set(xwsse);

                http_clone
                    .request::<CreateContactResponse>(Method::Post, url, Some(request_body), Some(headers))
                    .map_err(|e| e.context(Error::HttpClient).into())
            }).and_then(|response| {
                response
                    .extract_cteated_id()
                    .map_err(|e| e.context(format_err!("Error in emarsys response. Response: {:?}", response)).into())
            }).then(|res| match res {
                Ok(id) => Ok(id),
                Err(err) => {
                    error!("{}", err);
                    Err(err)
                }
            }).map(move |emarsys_id| CreatedContact { emarsys_id, user_id });
        Box::new(res)
    }
}

const EMAIL_FIELD: &'static str = "3";

#[derive(Debug, Clone, Serialize)]
struct CreateContactRequest {
    key_id: String,
    contacts: Vec<serde_json::Value>,
}

/// https://dev.emarsys.com/v2/contacts/create-contacts
#[derive(Debug, Clone, Deserialize)]
struct CreateContactResponse {
    /// The Emarsys response code
    /// https://dev.emarsys.com/v2/response-codes/error-codes
    #[serde(rename = "replyCode")]
    reply_code: Option<i64>,
    /// The summary of the response
    #[serde(rename = "replyText")]
    reply_text: Option<String>,
    /// The requested data.
    data: Option<CreateContactResponseData>,
}

/// The requested data.
#[derive(Debug, Clone, Deserialize)]
struct CreateContactResponseData {
    /// List of contact identifiers (id) of successfully created contacts.
    ids: Option<Vec<i32>>,
    /// List of errors during creating contacts.
    errors: Option<serde_json::Value>,
}

#[derive(Clone)]
pub struct ApiSecretKey(String);

#[derive(Clone)]
pub struct Nonce(Uuid);

#[derive(Debug, Clone)]
struct Signature {
    pub username_token: String,
    pub nonce: Nonce,
    pub timestamp: DateTime<Utc>,
    pub password_digest: PasswordDigest,
}

#[derive(Debug, Clone)]
struct PasswordDigest {
    pub nonce: Nonce,
    pub timestamp: DateTime<Utc>,
    pub api_secret_key: ApiSecretKey,
}

impl CreateContactResponse {
    fn extract_cteated_id(&self) -> Result<EmarsysId, FailureError> {
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

impl Signature {
    fn new(username_token: String, api_secret_key: String) -> Signature {
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

    fn calculate(&self) -> String {
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
    fn calculate(&self) -> String {
        let hashed_string = format!("{}{}{}", self.nonce, date_time_iso8601(self.timestamp), self.api_secret_key.0);
        let sha1_hash = Sha1::digest(hashed_string.as_bytes());
        base64::encode(&bytes_to_hex(&sha1_hash))
    }
}

impl From<CreateContactPayload> for CreateContactRequest {
    fn from(data: CreateContactPayload) -> CreateContactRequest {
        CreateContactRequest {
            key_id: EMAIL_FIELD.to_string(),
            contacts: vec![serde_json::json!({EMAIL_FIELD: data.email})],
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
