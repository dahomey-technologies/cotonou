use crate::{
    http::HttpClient,
    steam::{Error, SteamResponse, SteamParams, SteamId},
};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AuthenticateUserTicketResult {
    #[serde(rename = "steamid")]
    pub steam_id: SteamId,
    #[serde(rename = "ownersteamid")]
    pub owner_steam_id: String,
    #[serde(rename = "vacbanned")]
    pub vac_banned: bool,
    #[serde(rename = "publisherbanned")]
    pub publisher_banned: bool,
}

pub struct SteamUserAuthClient {
    pub http_client: HttpClient,
}

impl SteamUserAuthClient {
    pub fn new(http_client: HttpClient) -> Self {
        Self { http_client }
    }

    /// https://partner.steamgames.com/doc/webapi/ISteamUserAuth#AuthenticateUserTicket
    pub async fn authenticate_user_ticket(
        &self,
        key: &str,
        app_id: u32,
        ticket: &str,
        identity: &str
    ) -> Result<AuthenticateUserTicketResult, Error> {
        let url = format!("https://api.steampowered.com/ISteamUserAuth/AuthenticateUserTicket/v1/?key={key}&format=json&appId={app_id}&ticket={ticket}&identity={identity}");
        let response: SteamResponse<SteamParams<AuthenticateUserTicketResult>> =
            self.http_client.get(&url).await?;

        match response.response {
            SteamParams::Params(result) => Ok(result),
            SteamParams::Error(error) => Err(Error::SteamError(error)),
        }
    }
}

