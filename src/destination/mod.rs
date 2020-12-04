use crate::extension::log::CloudWatchLog;
use crate::parser::Parser;
use crate::LogDest;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

mod loggly;

pub trait Destination {
    fn handle_logs(&self, logs: Vec<CloudWatchLog>) -> Result<()>;
}

pub fn get_default() -> Result<LogDest> {
    let token = std::env::var("LOGGLY_TOKEN").unwrap();
    let tag = std::env::var("LOGGLY_TAG").unwrap();

    Ok(Arc::new(RwLock::new(
        loggly::Loggly::builder()
            .with_token(token)
            .with_tag(tag)
            .with_parser(Parser)
            .build()?,
    )))
}

#[cfg(test)]
mod custom;
#[cfg(test)]
pub fn get_test_destination() -> Result<LogDest> {
    Ok(Arc::new(RwLock::new(custom::Custom::new(Parser))))
}
