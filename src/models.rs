use byte_chunk::SizeInBytes;
use serde::Deserialize;
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

#[derive(Debug, Clone)]
pub struct Log {
    pub record: serde_json::Value,
    pub attempts: usize,
}

impl Log {
    pub fn new(record: serde_json::Value) -> Self {
        Log {
            record,
            attempts: 0,
        }
    }
}

impl ToString for Log {
    fn to_string(&self) -> String {
        self.record.to_string()
    }
}

impl SizeInBytes for Log {
    fn bytes_size(&self) -> usize {
        self.to_string().len()
    }
}
