use crate::database::GenericDAL;
use std::sync::Arc;

pub struct ProfileManager {
    pub is_development: bool,
    pub generic_dal: Arc<GenericDAL>,
}
