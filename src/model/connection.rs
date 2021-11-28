use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::model::RuleType;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct Connections {
    pub connections: Vec<Connection>,
    pub download_total: u64,
    pub upload_total: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub id: String,
    pub upload: u64,
    pub download: u64,
    pub metadata: Metadata,
    pub rule: RuleType,
    pub rule_payload: String,
    pub start: DateTime<Utc>,
    pub chains: Vec<String>,
}

impl Connection {
    pub fn up_speed(&self) -> Option<u64> {
        let elapsed = (Utc::now() - self.start).num_seconds();
        if elapsed <= 0 {
            None
        } else {
            Some(self.upload / elapsed as u64)
        }
    }

    pub fn down_speed(&self) -> Option<u64> {
        let elapsed = (Utc::now() - self.start).num_seconds();
        if elapsed <= 0 {
            None
        } else {
            Some(self.download / elapsed as u64)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    #[serde(rename = "type")]
    pub connection_type: String,

    #[serde(rename = "sourceIP")]
    pub source_ip: String,
    pub source_port: String,

    #[serde(rename = "destinationIP")]
    pub destination_ip: String,
    pub destination_port: String,
    pub host: String,
    pub network: String,
}
