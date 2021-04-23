use crate::models::RawCloudWatchLog;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

#[allow(dead_code)]
pub const DEFAULT_TIMEOUT: u64 = 500;

#[async_trait]
pub trait LogHandler {
    async fn handle_logs(&self, logs: Vec<RawCloudWatchLog>) -> Result<()>;
    fn get_name(&self) -> &str;
}

pub type Handler = Arc<RwLock<dyn LogHandler + Sync + Send>>;

cfg_if::cfg_if! {
    if #[cfg(feature = "loggly")] {
        mod loggly;
        use loggly::build_default;
    } else if #[cfg(feature = "logzio")] {
        mod logzio;
        use logzio::build_default;
    } else {
        mod custom;
        use custom::build_default;
        #[cfg(test)]
        pub fn get_test_destination() -> Result<Handler> {
            get_default()
        }
    }
}

pub fn get_default() -> Result<Handler> {
    let handler = build_default()?;
    Ok(Arc::new(RwLock::new(handler)))
}
