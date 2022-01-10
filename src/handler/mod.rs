use crate::models::Log;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

const DEFAULT_TIMEOUT: u64 = 1000;

#[derive(Debug)]
pub struct FailedToSendLogsError {
    pub logs: Vec<Log>,
}

impl From<Vec<Log>> for FailedToSendLogsError {
    fn from(logs: Vec<Log>) -> Self {
        FailedToSendLogsError { logs }
    }
}

pub type LogHandlerResponse = Result<(), FailedToSendLogsError>;

#[async_trait]
pub trait LogHandler {
    async fn handle_logs(&self, logs: Vec<Log>) -> LogHandlerResponse;
}

pub type Handler = Arc<RwLock<dyn LogHandler + Sync + Send>>;

cfg_if::cfg_if! {
    if #[cfg(feature = "loggly")] {
        mod loggly;
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
                    .build()?,
            )))
        }
    } else if #[cfg(feature = "logzio")] {
        mod logzio;
        pub fn get_default() -> Result<Handler> {
            let token = std::env::var("LOGZIO_TOKEN").unwrap();
            let host = std::env::var("LOGZIO_HOST").unwrap();
            let timeout: Option<u64> = match std::env::var("LOGZIO_TIMEOUT") {
                Ok(data) => match data.parse() {
                    Ok(0) => {
                        println!("LOGZIO_TIMEOUT set to Infinite");
                        None
                    }
                    Ok(t) => {
                        println!("LOGZIO_TIMEOUT set to {}ms", &t);
                        Some(t)
                    }
                    Err(_) => {
                        println!("LOGZIO_TIMEOUT: Cannot be parsed from {}", data);
                        Some(DEFAULT_TIMEOUT)
                    }
                },
                Err(_) => Some(DEFAULT_TIMEOUT),
            };

            Ok(Arc::new(RwLock::new(
                logzio::Logzio::builder()
                    .with_token(token)
                    .with_host(host)
                    .with_timeout(timeout)
                    .build()?,
            )))
        }
    } else if #[cfg(feature = "firehose")] {
        mod firehose;
        pub fn get_default() -> Result<Handler> {
            let stream = std::env::var("WOODCHUCK_FIREHOSE_TARGET").unwrap();
            let metadata = serde_json::from_str((std::env::var("WOODCHUCK_FIREHOSE_METADATA")?).as_ref())?;
            Ok(Arc::new(RwLock::new(
                firehose::Firehose::new(stream, metadata)
            )))
        }
    } else {
        mod custom;
        pub fn get_default() -> Result<Handler> {
            Ok(Arc::new(RwLock::new(custom::Custom::new())))
        }
    }
}
