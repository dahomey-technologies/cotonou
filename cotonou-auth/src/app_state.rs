use crate::error::Error;
use axum::extract::FromRef;
use cotonou_common::{
    profile::{AccountManager, CoreProfileManager},
    database::{GenericDAL, IdGeneratorDAL},
    mongo_db::MongoDbConfig,
    steam::{SteamMicroTxnClient, SteamUserAuthClient, SteamUserClient}, http::HttpClient,
};
use hyper_tls::HttpsConnector;
use std::sync::Arc;

#[derive(Clone, FromRef)]
pub struct AppState {
    is_development: bool,
    account_manager: Arc<AccountManager>,
    core_profile_manager: Arc<CoreProfileManager>,
    steam_user_auth_client: Arc<SteamUserAuthClient>,
    steam_user_client: Arc<SteamUserClient>,
    steam_micro_tnx_client: Arc<SteamMicroTxnClient>,
}

impl AppState {
    pub async fn new() -> Result<AppState, Error> {
        let generic_dal = GenericDAL::initialize(&MongoDbConfig {
            connection_string: "mongodb://mongo:27017/test".to_owned(),
        })
        .await?;

        let id_generator_dal = IdGeneratorDAL {
            generic_dal: generic_dal.clone(),
        };
        let account_manager = AccountManager {
            id_generator_dal,
            generic_dal: generic_dal.clone(),
        };
        let core_profile_manager = CoreProfileManager { generic_dal };
        let http_client = HttpClient::new(
            hyper::client::Client::builder().build::<_, hyper::Body>(HttpsConnector::new()),
        );
        let steam_user_auth_client = SteamUserAuthClient::new(http_client.clone());
        let steam_user_client = SteamUserClient::new(http_client.clone());
        let steam_micro_tnx_client = SteamMicroTxnClient::new(http_client);

        Ok(Self {
            is_development: true,
            account_manager: Arc::new(account_manager),
            core_profile_manager: Arc::new(core_profile_manager),
            steam_user_auth_client: Arc::new(steam_user_auth_client),
            steam_user_client: Arc::new(steam_user_client),
            steam_micro_tnx_client: Arc::new(steam_micro_tnx_client),
        })
    }
}
