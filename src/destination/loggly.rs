use crate::destination::Destination;
use crate::extension::log::CloudWatchLog;
use crate::parser::Parser;
use anyhow::{ensure, Error, Result};
use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;

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

impl Destination for Loggly {
    fn handle_logs(&self, cloudwatch_logs: Vec<CloudWatchLog>) -> Result<()> {
        let logs = self.parser.parse(cloudwatch_logs);

        let payload = logs
            .into_iter()
            .map(|log| serde_json::to_string(&log))
            .flatten()
            .collect::<Vec<String>>()
            .join("\n");

        let res = self
            .client
            .post(&self.url)
            .header(CONTENT_TYPE, "text/plain")
            .body(payload)
            .send()?;

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
}

impl LogglyBuilder {
    pub fn new() -> Self {
        LogglyBuilder {
            tag: None,
            token: None,
            parser: None,
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

    pub fn build(self) -> Result<Loggly> {
        match self {
            Self {
                tag: Some(tag),
                token: Some(token),
                parser: Some(parser),
            } => Ok(Loggly {
                url: format!("http://logs-01.loggly.com/bulk/{}/tag/{}/", token, tag),
                parser,
                client: Client::builder().timeout(None).build()?,
            }),
            Self { token: None, .. } => Err(Error::msg("Token Required")),
            Self { tag: None, .. } => Err(Error::msg("Tag Required")),
            Self { parser: None, .. } => Err(Error::msg("Parser Required")),
        }
    }
}