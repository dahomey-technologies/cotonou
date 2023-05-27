use crate::{jwt_claims::JwtRole, models::ProfileId};

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
