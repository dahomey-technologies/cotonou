mod authentication;
mod dal;
pub mod matchmaking;
pub mod models;
pub mod mongo;
pub mod notifications;
mod profile;
pub mod redis;

pub use authentication::*;
pub use dal::*;
pub use profile::*;

pub fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
