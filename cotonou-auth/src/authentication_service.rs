use crate::error::Error;
use axum::{extract::State, http::HeaderMap, Extension, Json};
use cotonou_common::{
    account_entity::AccountEntity,
    account_manager::AccountManager,
    core_profile_entity::CoreProfileEntity,
    core_profile_manager::CoreProfileManager,
    get_authorization,
    jwt_claims::{JwtClaims, JwtRole},
    matchmaking::game_server::GameServerId,
    models::HostingEnvironment,
    steam::{self, SteamId, SteamMicroTxnClient, SteamUserAuthClient, SteamUserClient},
    unix_now,
    user::User,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::Serialize;
use std::{
    result,
    sync::Arc,
    time::{Duration, SystemTime},
};

pub const STEAM_WEB_API_KEY: &str = "";
pub const STEAM_APP_ID: u32 = 0;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticationInfo {
    pub auth_token: String,
}

pub type Result<T> = result::Result<T, Error>;

pub async fn authenticate(
    State(hosting_environment): State<HostingEnvironment>,
    State(account_manager): State<Arc<AccountManager>>,
    State(core_profile_manager): State<Arc<CoreProfileManager>>,
    State(steam_user_auth_client): State<Arc<SteamUserAuthClient>>,
    State(steam_user_client): State<Arc<SteamUserClient>>,
    State(steam_micro_tnx_client): State<Arc<SteamMicroTxnClient>>,
    headers: HeaderMap,
) -> Result<Json<AuthenticationInfo>> {
    let (scheme, credentials) = get_authorization(&headers).ok_or(Error::NoAuthorizeHeader)?;

    let subject: String;
    let role: JwtRole;
    let country_code: String;
    let currency: String;

    match scheme {
        "nul" => {
            if hosting_environment != HostingEnvironment::Dev {
                return Err(Error::Unauthorized);
            }

            let display_name = credentials;
            let core_profile = get_or_create_account_entity(
                account_manager,
                core_profile_manager,
                &format!("nul-{display_name}"),
                display_name,
            )
            .await?;
            subject = core_profile.id.to_string();
            role = JwtRole::Player;
            country_code = "FR".to_owned();
            currency = "EUR".to_owned();
        }
        "stm" => {
            let (steam_id, vac_banned, publisher_banned) = authenticate_steam(
                steam_user_auth_client,
                steam_user_client.clone(),
                credentials,
            )
            .await?;

            if vac_banned || publisher_banned {
                return Err(Error::Forbidden);
            }

            let Some(steam_player_summary) = steam_user_client
                .get_player_summary(STEAM_WEB_API_KEY, steam_id).await? else {
                return Err(Error::Unauthorized);
            };

            let display_name = &steam_player_summary.persona_name;

            let user_info = steam_micro_tnx_client
                .get_user_info(
                    hosting_environment == HostingEnvironment::Dev,
                    STEAM_WEB_API_KEY,
                    STEAM_APP_ID,
                    steam_id,
                )
                .await?;
            country_code = user_info.country;
            currency = user_info.currency;

            let core_profile = get_or_create_account_entity(
                account_manager,
                core_profile_manager,
                &format!("stm-{steam_id}"),
                display_name,
            )
            .await?;
            subject = core_profile.id.to_string();
            role = JwtRole::Player;
        }
        "srv" => {
            let server_id = authenticate_server(hosting_environment, credentials)?;

            subject = server_id.to_string();
            role = JwtRole::Server;
            country_code = String::from("");
            currency = String::from("");
        }
        _ => return Err(Error::InvalidScheme)?,
    };

    let auth_token = create_auth_token(&subject, role, &country_code, &currency)?;

    Ok(Json(AuthenticationInfo { auth_token }))
}

pub async fn keep_alive(Extension(user): Extension<User>) -> Result<Json<AuthenticationInfo>> {
    let token = create_auth_token(&user.subject, user.role, &user.country, &user.currency)?;

    Ok(Json(AuthenticationInfo { auth_token: token }))
}

fn create_auth_token(
    subject: &str,
    role: JwtRole,
    country: &str,
    currency: &str,
) -> Result<String> {
    let claims = JwtClaims::new(
        subject,
        SystemTime::now()
            .checked_add(Duration::from_secs(3600 * 4))
            .ok_or(Error::InvalidExpirationTime)?,
        role,
        country,
        currency,
    );

    let jwt = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret("secret".as_ref()),
    )?;

    Ok(jwt)
}

async fn authenticate_steam(
    steam_user_auth_client: Arc<SteamUserAuthClient>,
    steam_user_client: Arc<SteamUserClient>,
    ticket: &str,
) -> Result<(SteamId, bool, bool)> {
    let authenticate_user_ticket_result = match steam_user_auth_client
        .authenticate_user_ticket(STEAM_WEB_API_KEY, STEAM_APP_ID, ticket, "carpool-auth")
        .await
    {
        Ok(result) => result,
        // invalid ticket
        Err(steam::Error::SteamError(error)) if error.error_code == 101 => {
            return Err(Error::Unauthorized);
        }
        Err(e) => {
            return Err(e.into());
        }
    };

    let steam_id = authenticate_user_ticket_result.steam_id;

    let ownership_result = steam_user_client
        .check_app_ownership(STEAM_WEB_API_KEY, STEAM_APP_ID, steam_id)
        .await?;

    if !ownership_result.owns_app {
        return Err(Error::Unauthorized);
    }

    Ok((
        steam_id,
        authenticate_user_ticket_result.vac_banned,
        authenticate_user_ticket_result.publisher_banned,
    ))
}

fn authenticate_server(
    hosting_environment: HostingEnvironment,
    credentials: &str,
) -> Result<GameServerId> {
    match decode::<JwtClaims>(
        credentials,
        &DecodingKey::from_secret("server_secret".as_ref()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(jwt_claims) => {
            if jwt_claims.claims.expiration_time < unix_now() {
                return Err(Error::Unauthorized);
            }

            // if jwt_claims.claims.subject != jwt_claims.claims.issuer {
            //     return Err(Error::Unauthorized);
            // }

            if jwt_claims.claims.role != JwtRole::Server {
                return Err(Error::Unauthorized);
            }

            // TODO validate audience

            let server_id = GameServerId::try_parse(&jwt_claims.claims.subject)
                .ok_or_else(|| Error::Unauthorized)?;
            Ok(server_id)
        }
        Err(e) => {
            if hosting_environment == HostingEnvironment::Dev {
                let server_id =
                    GameServerId::try_parse(credentials).ok_or_else(|| Error::Unauthorized)?;
                Ok(server_id)
            } else {
                Err(e.into())
            }
        }
    }
}

async fn get_or_create_account_entity(
    account_manager: Arc<AccountManager>,
    core_profile_manager: Arc<CoreProfileManager>,
    platform_id: &str,
    display_name: &str,
) -> Result<CoreProfileEntity> {
    let account: AccountEntity;
    let mut core_profile: CoreProfileEntity;

    let account_result = account_manager.get_account_entity(platform_id).await?;
    let mut core_profile_result: Option<CoreProfileEntity> = None;

    match account_result {
        None => {
            account = account_manager.create_account_entity(platform_id).await?;
        }
        Some(account_inner) => {
            account = account_inner;
            core_profile_result = core_profile_manager
                .get_core_profile(account.profile_id)
                .await?;
        }
    }

    match core_profile_result {
        None => {
            core_profile = CoreProfileEntity::new(account.profile_id, platform_id, display_name);
            core_profile_manager
                .create_core_profile(&mut core_profile)
                .await?;
        }
        Some(core_profile_inner) => {
            core_profile = core_profile_inner;
            if core_profile.display_name != display_name && !display_name.is_empty() {
                core_profile_manager
                    .update_display_name(core_profile.id, display_name)
                    .await?;
            }
        }
    }
    Ok(core_profile)
}
