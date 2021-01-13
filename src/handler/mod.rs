use crate::models::RawCloudWatchLog;
use crate::parser::Parser;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

mod custom;
mod loggly;

const DEFAULT_TIMEOUT: u64 = 500;

#[async_trait]
pub trait LogHandler {
    async fn handle_logs(&self, logs: Vec<RawCloudWatchLog>) -> Result<()>;
}

pub type Handler = Arc<RwLock<dyn LogHandler + Sync + Send>>;

#[cfg(not(feature = "local"))]
pub fn get_default() -> Result<Handler> {
    let token = std::env::var("LOGGLY_TOKEN").unwrap();
    let tag = std::env::var("LOGGLY_TAG").unwrap();
    let timeout: Option<u64> = match std::env::var("LOGGLY_TIMEOUT") {
        Ok(data) => match data.parse() {
            Ok(0) => {
                println!("LOGGLY_TIMEOUT set to Infinite");
                None
            }
            Ok(t) => {
                println!("LOGGLY_TIMEOUT set to {}ms", &t);
                Some(t)
            }
            Err(_) => {
                println!("LOGGLY_TIMEOUT: Cannot be parsed from {}", data);
                Some(DEFAULT_TIMEOUT)
            }
        },
        Err(_) => Some(DEFAULT_TIMEOUT),
    };

    Ok(Arc::new(RwLock::new(
        loggly::Loggly::builder()
            .with_token(token)
            .with_tag(tag)
            .with_timeout(timeout)
            .with_parser(Parser)
            .build()?,
    )))
}

#[cfg(feature = "local")]
pub fn get_default() -> Result<Handler> {
    Ok(Arc::new(RwLock::new(custom::Custom::new(Parser))))
}

#[cfg(test)]
pub fn get_test_destination() -> Result<Handler> {
    Ok(Arc::new(RwLock::new(custom::Custom::new(Parser))))
}
