use std::fmt;

use chrono::{DateTime, Utc};
use failure::Error as FailureError;
use sha1::{Digest, Sha1};
use uuid::Uuid;

use stq_http::request_util::XWSSE;
use stq_types::{Alpha3, EmarsysId, UserId};

use errors::EmarsysError;
use errors::Error;

/// system fields
/// [https://dev.emarsys.com/v2/personalization/contact-system-fields]
pub const EMAIL_FIELD: &'static str = "3";
pub const FIRST_NAME_FIELD: &'static str = "1";
pub const LAST_NAME_FIELD: &'static str = "2";
pub const COUNTRY_FIELD: &'static str = "14";
pub const OPT_IN: &'static str = "31";
pub const OPT_IN_TRUE: i32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContactPayload {
    pub user_id: UserId,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: String,
    pub country: Option<Alpha3>,
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
    pub fn extract_created_id(&self) -> Result<EmarsysId, FailureError> {
        match self.reply_code {
            Some(0) => {
                let data = self.data.as_ref().ok_or(format_err!("data field is missing"))?;
                if let Some(ref _errors) = data.errors {
                    return Err(format_err!("Response data has errors"));
                }
                let ids = data.ids.as_ref().ok_or(format_err!("ids field is missing"))?;
                if ids.len() != 1 {
                    return Err(format_err!("Expected only one id"));
                }
                ids.first().map(|id| EmarsysId(*id)).ok_or(format_err!("Expected only one id"))
            }
            Some(code) => {
                let text = self.reply_text.clone().unwrap_or(format!("Reply code is {}", code));
                Err(failure::err_msg(text)
                    .context(Error::Emarsys(EmarsysError {
                        code,
                        text: self.reply_text.clone(),
                    }))
                    .into())
            }
            None => Err(format_err!("Missing reply code in response")),
        }
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
                EMAIL_FIELD: data.email,
                COUNTRY_FIELD: data.country.and_then(|country| get_country_code(&country)),
                OPT_IN: OPT_IN_TRUE,
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

/// [https://help.emarsys.com/hc/en-us/articles/115004634749-single-choice-fields-and-their-values]
fn get_country_code(country: &Alpha3) -> Option<i32> {
    match country.0.as_ref() {
        "AFG" => Some(1),
        "ALB" => Some(2),
        "DZA" => Some(3),
        "AND" => Some(4),
        "AGO" => Some(5),
        "ATG" => Some(6),
        "ARG" => Some(7),
        "ARM" => Some(8),
        "AUS" => Some(9),
        "AUT" => Some(10),
        "AZE" => Some(11),
        "BHS" => Some(12),
        "BHR" => Some(13),
        "BGD" => Some(14),
        "BRB" => Some(15),
        "BLR" => Some(16),
        "BEL" => Some(17),
        "BLZ" => Some(18),
        "BEN" => Some(19),
        "BTN" => Some(20),
        "BOL" => Some(21),
        "BIH" => Some(22),
        "BWA" => Some(23),
        "BRA" => Some(24),
        "BRN" => Some(25),
        "BGR" => Some(26),
        "BFA" => Some(27),
        "BUR" => Some(28),
        "BDI" => Some(29),
        "KHM" => Some(30),
        "CMR" => Some(31),
        "CAN" => Some(32),
        "CPV" => Some(33),
        "CAF" => Some(34),
        "TCD" => Some(35),
        "CHL" => Some(36),
        "CHN" => Some(37),
        "COL" => Some(38),
        "COM" => Some(39),
        "COG" => Some(40),
        "COD" => Some(41),
        "CRI" => Some(42),
        "CIV" => Some(43),
        "HRV" => Some(44),
        "CUB" => Some(45),
        "CYP" => Some(46),
        "CZE" => Some(47),
        "DNK" => Some(48),
        "DJI" => Some(49),
        "DMA" => Some(50),
        "DOM" => Some(51),
        "ECU" => Some(52),
        "EGY" => Some(53),
        "SLV" => Some(54),
        "GNQ" => Some(55),
        "ERI" => Some(56),
        "EST" => Some(57),
        "ETH" => Some(58),
        "FJI" => Some(59),
        "FIN" => Some(60),
        "FRA" => Some(61),
        "GAB" => Some(62),
        "GMB" => Some(63),
        "GEO" => Some(64),
        "DEU" => Some(65),
        "GHA" => Some(66),
        "GRC" => Some(67),
        "GRD" => Some(68),
        "GTM" => Some(69),
        "GIN" => Some(70),
        "GNB" => Some(71),
        "GUY" => Some(72),
        "HTI" => Some(73),
        "HND" => Some(74),
        "HUN" => Some(75),
        "ISL" => Some(76),
        "IND" => Some(77),
        "IDN" => Some(78),
        "IRN" => Some(79),
        "IRQ" => Some(80),
        "IRL" => Some(81),
        "ISR" => Some(82),
        "ITA" => Some(83),
        "JAM" => Some(84),
        "JPN" => Some(85),
        "JOR" => Some(86),
        "KAZ" => Some(87),
        "KEN" => Some(88),
        "KIR" => Some(89),
        "PRK" => Some(90),
        "KOR" => Some(91),
        "KWT" => Some(92),
        "KGZ" => Some(93),
        "LAO" => Some(94),
        "LVA" => Some(95),
        "LBN" => Some(96),
        "LSO" => Some(97),
        "LBR" => Some(98),
        "LBY" => Some(99),
        "LIE" => Some(100),
        "LTU" => Some(101),
        "LUX" => Some(102),
        "MKD" => Some(103),
        "MDG" => Some(104),
        "MWI" => Some(105),
        "MYS" => Some(106),
        "MDV" => Some(107),
        "MLI" => Some(108),
        "MLT" => Some(109),
        "MHL" => Some(110),
        "MRT" => Some(111),
        "MUS" => Some(112),
        "MEX" => Some(113),
        "FSM" => Some(114),
        "MDA" => Some(115),
        "MCO" => Some(116),
        "MNG" => Some(117),
        "MAR" => Some(118),
        "MOZ" => Some(119),
        "MMR" => Some(120),
        "NAM" => Some(121),
        "NRU" => Some(122),
        "NPL" => Some(123),
        "NLD" => Some(124),
        "NZL" => Some(125),
        "NIC" => Some(126),
        "NER" => Some(127),
        "NGA" => Some(128),
        "NOR" => Some(129),
        "OMN" => Some(130),
        "PAK" => Some(131),
        "PLW" => Some(132),
        "PAN" => Some(134),
        "PNG" => Some(135),
        "PRY" => Some(136),
        "PER" => Some(137),
        "PHL" => Some(138),
        "POL" => Some(139),
        "PRT" => Some(140),
        "QAT" => Some(141),
        "ROU" => Some(142),
        "RUS" => Some(143),
        "RWA" => Some(144),
        "KNA" => Some(145),
        "LCA" => Some(146),
        "VCT" => Some(147),
        "WSM" => Some(148),
        "SMR" => Some(149),
        "STP" => Some(150),
        "SAU" => Some(151),
        "SEN" => Some(152),
        "SRB" => Some(153),
        "SYC" => Some(154),
        "SLE" => Some(155),
        "SGP" => Some(156),
        "SVK" => Some(157),
        "SVN" => Some(158),
        "SLB" => Some(159),
        "SOM" => Some(160),
        "ZAF" => Some(161),
        "ESP" => Some(162),
        "LKA" => Some(163),
        "SDN" => Some(164),
        "SUR" => Some(165),
        "SWZ" => Some(166),
        "SWE" => Some(167),
        "CHE" => Some(168),
        "SYR" => Some(169),
        "TWN" => Some(170),
        "TJK" => Some(171),
        "TZA" => Some(172),
        "THA" => Some(173),
        "TGO" => Some(174),
        "TON" => Some(175),
        "TTO" => Some(176),
        "TUN" => Some(177),
        "TUR" => Some(178),
        "TKM" => Some(179),
        "TUV" => Some(180),
        "UGA" => Some(181),
        "UKR" => Some(182),
        "ARE" => Some(183),
        "GBR" => Some(184),
        "USA" => Some(185),
        "URY" => Some(186),
        "UZB" => Some(187),
        "VUT" => Some(188),
        "VAT" => Some(189),
        "VEN" => Some(190),
        "VNM" => Some(191),
        "ESH" => Some(192),
        "YEM" => Some(193),
        "SCG" => Some(194),
        "ZAR" => Some(195),
        "ZMB" => Some(196),
        "ZWE" => Some(197),
        "GRL" => Some(198),
        "VGB" => Some(199),
        "MNE" => Some(202),
        "GIB" => Some(203),
        "ANT" => Some(204),
        "HKG" => Some(205),
        "MAC" => Some(206),
        "TLS" => Some(258),
        "UNK" => Some(259),
        _ => None,
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
