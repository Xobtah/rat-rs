use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Default)]
pub struct UserAgent {
    pub name: String,
    pub version: String,
    pub os: String,
    pub hash: String,
}

impl FromStr for UserAgent {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut words = s.split_whitespace();
        let name_version = words.next().unwrap();
        let mut name_version = name_version.split("/");
        let name = name_version.next().unwrap();
        let version = name_version.next().unwrap();
        let os = words
            .next()
            .unwrap()
            .trim_start_matches("(")
            .trim_end_matches(")");
        let hash = words.next().unwrap();
        Ok(Self {
            name: name.to_string(),
            version: version.to_string(),
            os: os.to_string(),
            hash: hash.to_string(),
        })
    }
}

impl ToString for UserAgent {
    fn to_string(&self) -> String {
        format!("{}/{} ({}) {}", self.name, self.version, self.os, self.hash)
    }
}
