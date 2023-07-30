use super::{
    steam_response::{SteamParams, SteamParamsWithResult, SteamResponse},
    Error, SteamId,
};
use crate::http::HttpClient;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SteamMicroTxnGetUserInfoResult {
    pub state: String,
    pub country: String,
    pub currency: String,
    pub status: String,
}

/// https://partner.steamgames.com/doc/webapi/ISteamMicroTxn
pub struct SteamMicroTxnClient {
    pub http_client: HttpClient,
}

impl SteamMicroTxnClient {
    pub fn new(http_client: HttpClient) -> Self {
        SteamMicroTxnClient { http_client }
    }

    /// https://partner.steamgames.com/doc/webapi/ISteamMicroTxn#GetUserInfo
    pub async fn get_user_info(
        &self,
        is_development: bool,
        key: &str,
        app_id: u32,
        steam_id: SteamId,
    ) -> Result<SteamMicroTxnGetUserInfoResult, Error> {
        // https://partner.steamgames.com/doc/webapi/ISteamMicroTxnSandbox
        let api_interface = if is_development {
            "ISteamMicroTxnSandbox"
        } else {
            "ISteamMicroTxn"
        };
        let url = format!("https://api.steampowered.com/{api_interface}/GetUserInfo/v2?key={key}&format=json&appId={app_id}&steamId={steam_id}");
        let response: SteamResponse<SteamParamsWithResult<SteamMicroTxnGetUserInfoResult>> =
            self.http_client.get(&url).await?;

        match response.response.params {
            SteamParams::Params(result) => Ok(result),
            SteamParams::Error(error) => Err(Error::SteamError(error)),
        }
    }
}

