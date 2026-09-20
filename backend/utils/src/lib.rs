pub mod config;
pub mod extract;
pub mod http;

pub use config::Config;
pub use extract::{Session, UserId};
pub use http::reqwest_client;
use uuid::Uuid;

pub fn generate_uuid() -> Uuid {
    Uuid::now_v7()
}
