use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Default)]
pub struct Job {
    pub id: i32,
    pub agent_id: Option<i32>,
    pub issued_at: chrono::DateTime<chrono::Utc>,
    pub status: JobStatus,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub name: String,
    pub task: String,
    pub result: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub enum JobStatus {
    Issued,
    Running,
    Succeeded,
    Failed,
}

impl Default for JobStatus {
    fn default() -> Self {
        JobStatus::Issued
    }
}

impl ToString for JobStatus {
    fn to_string(&self) -> String {
        match self {
            JobStatus::Issued => "Issued".to_string(),
            JobStatus::Running => "Running".to_string(),
            JobStatus::Succeeded => "Succeeded".to_string(),
            JobStatus::Failed => "Failed".to_string(),
        }
    }
}

impl FromStr for JobStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Issued" => Ok(JobStatus::Issued),
            "Running" => Ok(JobStatus::Running),
            "Succeeded" => Ok(JobStatus::Succeeded),
            "Failed" => Ok(JobStatus::Failed),
            _ => Err(anyhow::anyhow!("Invalid JobStatus")),
        }
    }
}
