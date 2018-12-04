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
pub const COUNTRY_FIELD: &'static str = "14";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContactPayload {
    pub user_id: UserId,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: String,
    pub country: Option<String>,
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
                EMAIL_FIELD: data.email,
                COUNTRY_FIELD: data.country.and_then(|country| get_country_code(&country)),
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
fn get_country_code(country: &str) -> Option<i32> {
    match country {
        "Afghanistan" => Some(1),
        "Albania" => Some(2),
        "Algeria" => Some(3),
        "Andorra" => Some(4),
        "Angola" => Some(5),
        "Antigua and Barbuda" => Some(6),
        "Argentina" => Some(7),
        "Armenia" => Some(8),
        "Australia" => Some(9),
        "Austria" => Some(10),
        "Azerbaijan" => Some(11),
        "Bahamas" => Some(12),
        "Bahrain" => Some(13),
        "Bangladesh" => Some(14),
        "Barbados" => Some(15),
        "Belarus" => Some(16),
        "Belgium" => Some(17),
        "Belize" => Some(18),
        "Benin" => Some(19),
        "Bhutan" => Some(20),
        "Bolivia" => Some(21),
        "Bosnia and Herzegovina" => Some(22),
        "Botswana" => Some(23),
        "Brazil" => Some(24),
        "Brunei Darussalam" => Some(25),
        "Bulgaria" => Some(26),
        "Burkina Faso" => Some(27),
        "Burma" => Some(28),
        "Burundi" => Some(29),
        "Cambodia" => Some(30),
        "Cameroon" => Some(31),
        "Canada" => Some(32),
        "Cape Verde" => Some(33),
        "Central African Republic" => Some(34),
        "Chad" => Some(35),
        "Chile" => Some(36),
        "China" => Some(37),
        "Colombia" => Some(38),
        "Comoros" => Some(39),
        "Congo" => Some(40),
        "Congo, Democratic Republic of the" => Some(41),
        "Costa Rica" => Some(42),
        "Cote d’Ivoire" => Some(43),
        "Croatia" => Some(44),
        "Cuba" => Some(45),
        "Cyprus" => Some(46),
        "Czech Republic" => Some(47),
        "Denmark" => Some(48),
        "Djibouti" => Some(49),
        "Dominica" => Some(50),
        "Dominican Republic" => Some(51),
        "Ecuador" => Some(52),
        "Egypt" => Some(53),
        "El Salvador" => Some(54),
        "Equatorial Guinea" => Some(55),
        "Eritrea" => Some(56),
        "Estonia" => Some(57),
        "Ethiopia" => Some(58),
        "Fiji" => Some(59),
        "Finland" => Some(60),
        "France" => Some(61),
        "Gabon" => Some(62),
        "Gambia, The" => Some(63),
        "Georgia" => Some(64),
        "Germany" => Some(65),
        "Ghana" => Some(66),
        "Greece" => Some(67),
        "Grenada" => Some(68),
        "Guatemala" => Some(69),
        "Guinea" => Some(70),
        "Guinea-Bissau" => Some(71),
        "Guyana" => Some(72),
        "Haiti" => Some(73),
        "Honduras" => Some(74),
        "Hungary" => Some(75),
        "Iceland" => Some(76),
        "India" => Some(77),
        "Indonesia" => Some(78),
        "Iran" => Some(79),
        "Iraq" => Some(80),
        "Ireland" => Some(81),
        "Israel" => Some(82),
        "Italy" => Some(83),
        "Jamaica" => Some(84),
        "Japan" => Some(85),
        "Jordan" => Some(86),
        "Kazakhstan" => Some(87),
        "Kenya" => Some(88),
        "Kiribati" => Some(89),
        "Korea, North" => Some(90),
        "Korea, South" => Some(91),
        "Kuwait" => Some(92),
        "Kyrgyzstan" => Some(93),
        "Laos" => Some(94),
        "Latvia" => Some(95),
        "Lebanon" => Some(96),
        "Lesotho" => Some(97),
        "Liberia" => Some(98),
        "Libya" => Some(99),
        "Liechtenstein" => Some(100),
        "Lithuania" => Some(101),
        "Luxembourg" => Some(102),
        "Macedonia" => Some(103),
        "Madagascar" => Some(104),
        "Malawi" => Some(105),
        "Malaysia" => Some(106),
        "Maldives" => Some(107),
        "Mali" => Some(108),
        "Malta" => Some(109),
        "Marshall Islands" => Some(110),
        "Mauritania" => Some(111),
        "Mauritius" => Some(112),
        "Mexico" => Some(113),
        "Micronesia" => Some(114),
        "Moldova" => Some(115),
        "Monaco" => Some(116),
        "Mongolia" => Some(117),
        "Morocco" => Some(118),
        "Mozambique" => Some(119),
        "Myanmar" => Some(120),
        "Namibia" => Some(121),
        "Nauru" => Some(122),
        "Nepal" => Some(123),
        "The Netherlands" => Some(124),
        "New Zealand" => Some(125),
        "Nicaragua" => Some(126),
        "Niger" => Some(127),
        "Nigeria" => Some(128),
        "Norway" => Some(129),
        "Oman" => Some(130),
        "Pakistan" => Some(131),
        "Palau" => Some(132),
        "Panama" => Some(134),
        "Papua New Guinea" => Some(135),
        "Paraguay" => Some(136),
        "Peru" => Some(137),
        "Philippines" => Some(138),
        "Poland" => Some(139),
        "Portugal" => Some(140),
        "Qatar" => Some(141),
        "Romania" => Some(142),
        "Russia" => Some(143),
        "Rwanda" => Some(144),
        "St. Kitts and Nevis" => Some(145),
        "St. Lucia" => Some(146),
        "St. Vincent and The Grenadines" => Some(147),
        "Samoa" => Some(148),
        "San Marino" => Some(149),
        "São Tomé and Príncipe" => Some(150),
        "Saudi Arabia" => Some(151),
        "Senegal" => Some(152),
        "Serbia" => Some(153),
        "Seychelles" => Some(154),
        "Sierra Leone" => Some(155),
        "Singapore" => Some(156),
        "Slovakia" => Some(157),
        "Slovenia" => Some(158),
        "Solomon Islands" => Some(159),
        "Somalia" => Some(160),
        "South Africa" => Some(161),
        "Spain" => Some(162),
        "Sri Lanka" => Some(163),
        "Sudan" => Some(164),
        "Suriname" => Some(165),
        "Swaziland" => Some(166),
        "Sweden" => Some(167),
        "Switzerland" => Some(168),
        "Syria" => Some(169),
        "Taiwan" => Some(170),
        "Tajikistan" => Some(171),
        "Tanzania" => Some(172),
        "Thailand" => Some(173),
        "Togo" => Some(174),
        "Tonga" => Some(175),
        "Trinidad and Tobago" => Some(176),
        "Tunisia" => Some(177),
        "Turkey" => Some(178),
        "Turkmenistan" => Some(179),
        "Tuvalu" => Some(180),
        "Uganda" => Some(181),
        "Ukraine" => Some(182),
        "United Arab Emirates" => Some(183),
        "United Kingdom" => Some(184),
        "United States of America" => Some(185),
        "Uruguay" => Some(186),
        "Uzbekistan" => Some(187),
        "Vanuatu" => Some(188),
        "Vatican City" => Some(189),
        "Venezuela" => Some(190),
        "Vietnam" => Some(191),
        "Western Sahara" => Some(192),
        "Yemen" => Some(193),
        "Yugoslavia" => Some(194),
        "Zaire" => Some(195),
        "Zambia" => Some(196),
        "Zimbabwe" => Some(197),
        "Greenland" => Some(198),
        "Virgin Islands" => Some(199),
        "Canary Islands" => Some(201),
        "Montenegro" => Some(202),
        "Gibraltar" => Some(203),
        "Netherlands Antilles" => Some(204),
        "Hong Kong" => Some(205),
        "Macau" => Some(206),
        "East Timor" => Some(258),
        "Kosovo" => Some(259),
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
