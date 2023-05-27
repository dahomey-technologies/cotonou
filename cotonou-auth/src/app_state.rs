use axum::extract::FromRef;
use cotonou_common::{
    account_manager::AccountManager, core_profile_manager::CoreProfileManager,
    generic_dal::GenericDAL, id_generator_dal::IdGeneratorDAL, models::HostingEnvironment,
    mongo::mongo_config::MongoConfig,
};
use std::sync::Arc;

use crate::error::Error;

#[derive(Clone, FromRef)]
pub struct AppState {
    hosting_environment: HostingEnvironment,
    account_manager: Arc<AccountManager>,
    core_profile_manager: Arc<CoreProfileManager>,
}

impl AppState {
    pub async fn new() -> Result<AppState, Error> {
        let generic_dal = GenericDAL::initialize(&MongoConfig {
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

        Ok(Self {
            hosting_environment: HostingEnvironment::Dev,
            account_manager: Arc::new(account_manager),
            core_profile_manager: Arc::new(core_profile_manager),
        })
    }
}
