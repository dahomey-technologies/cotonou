use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum JwtRole {
    Player,
    Server,
    Admin,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    #[serde(rename = "sub")]
    pub subject: String,

    #[serde(rename = "exp")]
    pub expiration_time: u64,

    pub role: JwtRole,

    #[serde(rename = "ctry")]
    pub country: String,

    #[serde(rename = "ccy")]
    pub currency: String,
}

impl JwtClaims {
    pub fn new(
        subject: &str,
        expiration_time: SystemTime,
        role: JwtRole,
        country: &str,
        currency: &str,
    ) -> JwtClaims {
        JwtClaims {
            subject: subject.to_string(),
            expiration_time: Self::to_unix_time(expiration_time),
            role,
            country: country.to_string(),
            currency: currency.to_string(),
        }
    }

    fn to_unix_time(time: SystemTime) -> u64 {
        time.duration_since(UNIX_EPOCH).ok().unwrap().as_secs()
    }
}
