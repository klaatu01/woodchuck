use crate::handler::{LogHandler, LogHandlerResponse};
use crate::models::Log;
use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct Custom;

impl Custom {
    pub fn new() -> Self {
        Custom {}
    }
}

#[async_trait]
impl LogHandler for Custom {
    async fn handle_logs(&self, logs: Vec<Log>) -> LogHandlerResponse {
        for log in logs.iter() {
            log::debug!("PARSED = {:?}", log);
        }
        Ok(())
    }
}
