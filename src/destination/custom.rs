use crate::destination::Destination;
use crate::extension::log::CloudWatchLog;
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

impl Destination for Custom {
    fn handle_logs(&self, logs: Vec<CloudWatchLog>) -> Result<()> {
        let nd_logs = self.parser.parse(logs);
        for log in nd_logs.into_iter() {
            println!("PARSED = {:?}", log);
        }
        Ok(())
    }
}
