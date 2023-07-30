use crate::{authentication::JwtRole, types::ProfileId};

#[derive(Debug, Clone)]
pub struct User {
    pub subject: String,
    pub role: JwtRole,
    pub country: String,
    pub currency: String,
}

impl User {
    pub fn get_profile_id(&self) -> ProfileId {
        match self.role {
            JwtRole::Player => self.subject.parse().unwrap_or_default(),
            _ => ProfileId::default()
        }
    }
}
