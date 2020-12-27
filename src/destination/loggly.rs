use crate::destination::LogDestination;
use crate::models::RawCloudWatchLog;
use crate::parser::Parser;
use anyhow::{ensure, Error, Result};
use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Loggly {
    url: String,
    parser: Parser,
    client: Client,
}

impl Loggly {
    pub fn builder() -> LogglyBuilder {
        LogglyBuilder::new()
    }
}

impl LogDestination for Loggly {
    fn handle_logs(&self, cloudwatch_logs: Vec<RawCloudWatchLog>) -> Result<()> {
        let logs = self.parser.parse(cloudwatch_logs);

        let payload: String = logs
            .iter()
            .map(|log| log.to_string())
            .collect::<Vec<String>>()
            .join("\n");

        println!(
            "Sending {} logs to Loggly, payload length: {}",
            &logs.len(),
            &payload.len()
        );

        let res = self
            .client
            .post(&self.url)
            .header(CONTENT_TYPE, "text/plain")
            .body(payload)
            .send()?;

        println!("Response: Status:{}", &res.status());

        ensure!(
            res.status() == reqwest::StatusCode::OK,
            "Error Sending Logs"
        );

        Ok(())
    }
}

pub struct LogglyBuilder {
    token: Option<String>,
    tag: Option<String>,
    parser: Option<Parser>,
    timeout: Option<Duration>,
}

impl LogglyBuilder {
    pub fn new() -> Self {
        LogglyBuilder {
            tag: None,
            token: None,
            parser: None,
            timeout: None,
        }
    }

    pub fn with_tag(mut self, tag: String) -> Self {
        self.tag = Some(tag);
        self
    }

    pub fn with_token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    pub fn with_parser(mut self, parser: Parser) -> Self {
        self.parser = Some(parser);
        self
    }

    pub fn with_timeout(mut self, timeout: Option<u64>) -> Self {
        self.timeout = match timeout {
            Some(t) => Some(Duration::from_millis(t)),
            None => None,
        };
        self
    }

    pub fn build(self) -> Result<Loggly> {
        match self {
            Self {
                tag: Some(tag),
                token: Some(token),
                parser: Some(parser),
                timeout: _,
            } => Ok(Loggly {
                url: format!("http://logs-01.loggly.com/bulk/{}/tag/{}/", token, tag),
                parser,
                client: Client::builder().timeout(self.timeout).build()?,
            }),
            Self { token: None, .. } => Err(Error::msg("Token Required")),
            Self { tag: None, .. } => Err(Error::msg("Tag Required")),
            Self { parser: None, .. } => Err(Error::msg("Parser Required")),
        }
    }
}
