use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::TryFrom;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type LogQueue = Arc<RwLock<Vec<RawCloudWatchLog>>>;

pub fn new_log_queue() -> LogQueue {
    Arc::new(RwLock::new(Vec::new()))
}

#[derive(Default, Debug, Deserialize, Clone)]
pub struct RawCloudWatchLog {
    pub time: String,
    pub r#type: String,
    pub record: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct StructuredLog {
    pub timestamp: Option<String>,
    pub guid: Option<String>,
    pub level: Option<LogLevel>,
    pub data: Value,
}

#[derive(Debug, Serialize, PartialEq)]
pub enum LogLevel {
    #[serde(rename(serialize = "INFO"))]
    Info,
    #[serde(rename(serialize = "WARN"))]
    Warn,
    #[serde(rename(serialize = "ERROR"))]
    Error,
}

impl TryFrom<String> for LogLevel {
    type Error = anyhow::Error;
    fn try_from(level: String) -> Result<Self> {
        match level.as_str() {
            "INFO" => Ok(LogLevel::Info),
            "WARN" => Ok(LogLevel::Warn),
            "ERROR" => Ok(LogLevel::Error),
            _ => Err(Error::msg(format!("Unable to parse {} as LogLevel", level))),
        }
    }
}

#[derive(Debug)]
pub enum Log {
    Unformatted(StructuredLog),
    Formatted(serde_json::Value),
}

impl ToString for Log {
    fn to_string(&self) -> String {
        match self {
            Log::Unformatted(data) => serde_json::to_string(data).unwrap(),
            Log::Formatted(data) => data.to_string(),
        }
    }
}
