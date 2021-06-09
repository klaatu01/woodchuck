use crate::models::RawCloudWatchLog;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

#[allow(unused)]
use crate::parser::Parser;
#[allow(dead_code)]
const DEFAULT_TIMEOUT: u64 = 1000;

#[async_trait]
pub trait LogHandler {
    async fn handle_logs(&self, logs: Vec<RawCloudWatchLog>) -> Result<()>;
}

pub type Handler = Arc<RwLock<dyn LogHandler + Sync + Send>>;

#[allow(dead_code)]
fn parse_timeout(name: &str, default: u64) -> Option<u64> {
    match std::env::var(name) {
        Ok(data) => match data.parse() {
            Ok(0) => {
                println!("{} set to Infinite", name);
                None
            }
            Ok(t) => {
                println!("{} set to {}ms", name, &t);
                Some(t)
            }
            Err(_) => {
                println!("{} Cannot be parsed from {}", name, data);
                Some(default)
            }
        },
        Err(_) => Some(default),
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "loggly")] {
        mod loggly;
        pub fn get_default() -> Result<Handler> {
            let token = std::env::var("LOGGLY_TOKEN").unwrap();
            let tag = std::env::var("LOGGLY_TAG").unwrap();
            let timeout: Option<u64> = parse_timeout("LOGGLY_TIMEOUT", DEFAULT_TIMEOUT);

            Ok(Arc::new(RwLock::new(
                loggly::Loggly::builder()
                    .with_token(token)
                    .with_tag(tag)
                    .with_timeout(timeout)
                    .with_parser(Parser)
                    .build()?,
            )))
        }
    } else if #[cfg(feature = "logzio")] {
        mod logzio;
        pub fn get_default() -> Result<Handler> {
            let token = std::env::var("LOGZIO_TOKEN").unwrap();
            let host = std::env::var("LOGZIO_HOST").unwrap();
            let timeout: Option<u64> = parse_timeout("LOGZIO_TIMEOUT", DEFAULT_TIMEOUT);

            Ok(Arc::new(RwLock::new(
                logzio::Logzio::builder()
                    .with_token(token)
                    .with_host(host)
                    .with_timeout(timeout)
                    .with_parser(Parser)
                    .build()?,
            )))
        }
    } else if #[cfg(feature = "firehose")] {
        mod firehose;
        pub fn get_default() -> Result<Handler> {
            let delivery_stream_name = std::env::var("FIREHOSE_NAME").unwrap();
            let tag = std::env::var("FIREHOSE_TAG").unwrap();
            Ok(Arc::new(RwLock::new(firehose::Firehose::new(delivery_stream_name, tag))))
        }
    } else {
        mod custom;
        pub fn get_default() -> Result<Handler> {
            Ok(Arc::new(RwLock::new(custom::Custom::new(Parser))))
        }
        #[cfg(test)]
        pub fn get_test_destination() -> Result<Handler> {
            Ok(Arc::new(RwLock::new(custom::Custom::new(Parser))))
        }
    }
}
