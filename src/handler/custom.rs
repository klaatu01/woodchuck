use crate::handler::LogHandler;
use crate::models::RawCloudWatchLog;
use crate::parser::Parser;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Custom {
    parser: Parser,
}

impl Custom {
    pub fn new(parser: Parser) -> Self {
        Custom { parser }
    }
}

impl LogHandler for Custom {
    fn handle_logs(&self, logs: Vec<RawCloudWatchLog>) -> Result<()> {
        let nd_logs = self.parser.parse(logs);
        for log in nd_logs.into_iter() {
            println!("PARSED = {:?}", log);
        }
        Ok(())
    }
}
