use crate::handler::LogHandler;
use crate::models::RawCloudWatchLog;
use crate::parser::Parser;
use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct Custom {
    parser: Parser,
}

impl Custom {
    pub fn new(parser: Parser) -> Self {
        Custom { parser }
    }
}

#[async_trait]
impl LogHandler for Custom {
    async fn handle_logs(&self, logs: Vec<RawCloudWatchLog>) -> Result<()> {
        log::debug!("PARSED = {:?}", &logs);
        let nd_logs = self.parser.parse(logs);
        for log in nd_logs.into_iter() {
            log::debug!("PARSED = {:?}", log);
        }
        Ok(())
    }
}
