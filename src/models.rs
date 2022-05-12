use anyhow::{Error, Result};
use byte_chunk::SizeInBytes;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::TryFrom;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type LogQueue = Arc<RwLock<Vec<Log>>>;

pub fn new_log_queue() -> LogQueue {
    Arc::new(RwLock::new(Vec::new()))
}

#[derive(Default, Debug, Deserialize, Clone)]
pub struct RawCloudWatchLog {
    pub time: String,
    pub r#type: String,
    pub record: serde_json::Value,
}

#[derive(Debug, Serialize, Clone)]
pub struct StructuredLog {
    pub timestamp: Option<String>,
    pub guid: Option<String>,
    pub level: Option<LogLevel>,
    pub data: Value,
}

#[derive(Debug, Serialize, PartialEq, Clone)]
pub enum LogLevel {
    #[serde(rename(serialize = "INFO"))]
    Info,
    #[serde(rename(serialize = "WARN"))]
    Warn,
    #[serde(rename(serialize = "ERROR"))]
    Error,
    #[serde(rename(serialize = "TRACE"))]
    Trace,
    #[serde(rename(serialize = "CRITICAL"))]
    Critical,
    #[serde(rename(serialize = "DEBUG"))]
    Debug,
}

impl TryFrom<String> for LogLevel {
    type Error = anyhow::Error;
    fn try_from(level: String) -> Result<Self> {
        match level.as_str() {
            "INFO" => Ok(LogLevel::Info),
            "WARN" => Ok(LogLevel::Warn),
            "ERROR" => Ok(LogLevel::Error),
            "TRACE" => Ok(LogLevel::Trace),
            "CRITICAL" => Ok(LogLevel::Critical),
            "DEBUG" => Ok(LogLevel::Debug),
            _ => Err(Error::msg(format!("Unable to parse {} as LogLevel", level))),
        }
    }
}

#[derive(Debug, Clone)]
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

impl SizeInBytes for Log {
    fn bytes_size(&self) -> usize {
        self.to_string().len()
    }
}
