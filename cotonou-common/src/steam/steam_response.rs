use serde::Deserialize;
use std::fmt;

#[derive(Deserialize, Debug)]
pub struct SteamResponse<T> {
    pub response: T,
}

#[derive(Deserialize, Debug, PartialEq)]
pub enum SteamResultCode {
    OK,
    Failure,
}

#[derive(Deserialize, Debug, Default, thiserror::Error)]
pub struct SteamError {
    #[serde(rename="errorcode")]
    pub error_code: u32,
    #[serde(rename="errordesc")]
    pub error_desc: String,
}

impl fmt::Display for SteamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("({}) {}", self.error_code, self.error_desc))
    }
}

#[derive(Deserialize, Debug)]
pub enum SteamParams<T> {
    #[serde(rename="params")]
    Params(T),
    #[serde(rename="error")]
    Error(SteamError),
}

#[derive(Deserialize, Debug)]
pub struct SteamParamsWithResult<T> {
    pub result: SteamResultCode,
    #[serde(flatten)]
    pub params: SteamParams<T>,
}
