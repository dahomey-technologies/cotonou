use crate::{
    deserialize_iso_datetime, deserialize_unix_datetime,
    http::HttpClient,
    steam::{Error, SteamId, SteamResponse},
};
use serde::Deserialize;
use serde_repr::Deserialize_repr;
use std::fmt::Write;

#[derive(Deserialize, Debug)]
pub struct AppOwnershipResult {
    #[serde(rename = "ownsapp")]
    pub owns_app: bool,
    pub permanent: bool,
    #[serde(deserialize_with = "deserialize_iso_datetime")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "ownersteamid")]
    pub owner_steam_id: String,
    #[serde(rename = "sitelicense")]
    pub site_license: bool,
    #[serde(rename = "timedtrial")]
    pub timed_trial: bool,
    pub result: String,
}

/// https://partner.steamgames.com/doc/api/ISteamFriends#EPersonaState
/// https://steam.readthedocs.io/en/latest/api/steam.enums.html#steam.enums.common.EPersonaState
#[derive(Deserialize_repr, Debug)]
#[repr(u8)]
pub enum SteamPersonaState {
    Offline = 0,
    Online = 1,
    Busy = 2,
    Away = 3,
    Snooze = 4,
    LookingToTrade = 5,
    LookingToPlay = 6,
    Invisible = 7,
}

#[derive(Deserialize_repr, Debug)]
#[repr(u8)]
pub enum SteamCommunityVisibleState {
    Private = 1,
    FriendsOnly = 2,
    Public = 3,
}

#[derive(Deserialize_repr, Debug)]
#[repr(u8)]
pub enum SteamProfileState {
    Configured = 1,
}

#[derive(Deserialize_repr, Debug)]
#[repr(u8)]
pub enum SteamCommentPermission {
    None = 0,
    AllowPublicComments = 1,
}

#[derive(Deserialize, Debug)]
pub struct SteamPlayerSummary {
    #[serde(rename = "steamid")]
    pub steam_id: SteamId,

    #[serde(rename = "personaname")]
    pub persona_name: String,

    #[serde(rename = "profileurl", default)]
    pub profile_url: String,

    #[serde(rename = "avatar", default)]
    pub avatar: String,

    #[serde(rename = "avatarmedium", default)]
    pub avatar_medium: String,

    #[serde(rename = "avatarfull", default)]
    pub avatar_full: String,

    #[serde(rename = "avatarhash", default)]
    pub avatar_hash: String,

    #[serde(rename = "personastate")]
    pub persona_state: SteamPersonaState,

    /// Bit mask
    #[serde(rename = "personastateflags", default)]
    pub persona_state_flags: u32,

    #[serde(rename = "communityvisibilitystate")]
    pub community_visibility_state: SteamCommunityVisibleState,

    #[serde(rename = "profilestate")]
    pub profile_state: SteamProfileState,

    #[serde(rename = "primaryclanid", default)]
    pub primary_clan_id: SteamId,

    #[serde(rename = "timecreated", default)]
    #[serde(deserialize_with = "deserialize_unix_datetime")]
    pub time_created: chrono::DateTime<chrono::Utc>,

    #[serde(rename = "loccountrycode", default)]
    pub loc_country_code: String,

    #[serde(rename = "locstatecode", default)]
    pub loc_state_code: String,

    #[serde(rename = "loccityid", default)]
    pub loc_city_id: u32,
}
pub struct SteamUserClient {
    pub http_client: HttpClient,
}

impl SteamUserClient {
    pub fn new(http_client: HttpClient) -> Self {
        SteamUserClient { http_client }
    }

    /// https://partner.steamgames.com/doc/webapi/ISteamUser#CheckAppOwnership
    pub async fn check_app_ownership(
        &self,
        key: &str,
        app_id: u32,
        steam_id: SteamId,
    ) -> Result<AppOwnershipResult, Error> {
        #[derive(Deserialize)]
        struct AppOwnershipResponse {
            #[serde(rename = "appownership")]
            pub app_ownership: AppOwnershipResult,
        }

        let url = format!("https://api.steampowered.com/ISteamUser/CheckAppOwnership/v2?key={key}&format=json&appId={app_id}&steamId={steam_id}");
        let response: AppOwnershipResponse = self.http_client.get(&url).await?;
        Ok(response.app_ownership)
    }

    /// https://partner.steamgames.com/doc/webapi/ISteamUser#GetPlayerSummaries
    /// https://developer.valvesoftware.com/wiki/Steam_Web_API#GetPlayerSummaries_.28v0002.29
    pub async fn get_player_summaries<I: IntoIterator<Item = SteamId>>(
        &self,
        key: &str,
        steam_ids: I,
    ) -> Result<Vec<SteamPlayerSummary>, Error> {
        #[derive(Deserialize)]
        struct GetPlayerSummariesResponse {
            #[serde(rename = "players")]
            pub players: Vec<SteamPlayerSummary>,
        }

        // concatenate steam ids into a string, separated with comas
        let mut it = steam_ids.into_iter();
        let first = it.next().map(|f| f.to_string()).unwrap_or_default();
        let steam_ids_str = it.fold(first, |mut acc, id| {
            write!(acc, ",{id}").expect("writing in a string should not fail");
            acc
        });

        let url = format!("https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/?key={key}&steamids={steam_ids_str}");
        let response: SteamResponse<GetPlayerSummariesResponse> =
            self.http_client.get(&url).await?;
        Ok(response.response.players)
    }

    /// https://partner.steamgames.com/doc/webapi/ISteamUser#GetPlayerSummaries
    pub async fn get_player_summary(
        &self,
        key: &str,
        steam_id: SteamId,
    ) -> Result<Option<SteamPlayerSummary>, Error> {
        Ok(self
            .get_player_summaries(key, [steam_id])
            .await?
            .into_iter()
            .next())
    }
}
