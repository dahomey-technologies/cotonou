#[typetag::serde(tag = "type")]
pub trait Notification: Send + Sync {}
