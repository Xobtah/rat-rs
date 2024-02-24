use common::UserAgent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Agent {
    pub id: i32,
    pub mac_address: String,
    pub user_agent: UserAgent,
    pub sha256: String,
    pub name: String,
    pub registered_at: chrono::DateTime<chrono::Utc>,
    pub last_seen_at: chrono::DateTime<chrono::Utc>,
}
