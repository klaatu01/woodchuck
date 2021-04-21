use crate::handler::LogHandler;
use crate::models::RawCloudWatchLog;
use crate::parser::Parser;
use anyhow::{ensure, Error, Result};
use async_trait::async_trait;
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Logzio {
    url: String,
    parser: Parser,
    client: Client,
}

impl Logzio {
    pub fn builder() -> LogzioBuilder {
        LogzioBuilder::new()
    }
}

#[async_trait]
impl LogHandler for Logzio {
    async fn handle_logs(&self, cloudwatch_logs: Vec<RawCloudWatchLog>) -> Result<()> {
        let logs = self.parser.parse(cloudwatch_logs);

        let payload: String = logs
            .iter()
            .map(|log| log.to_string())
            .collect::<Vec<String>>()
            .join("\n");

        log::debug!(
            "Sending {} logs to Logzio, payload length: {}",
            &logs.len(),
            &payload.len()
        );

        let res = self
            .client
            .post(&self.url)
            .header(CONTENT_TYPE, "text/plain")
            .body(payload)
            .send()
            .await?;

        println!("Response: Status:{}", &res.status());

        ensure!(
            res.status() == reqwest::StatusCode::OK,
            "Error Sending Logs"
        );

        Ok(())
    }
}

pub struct LogzioBuilder {
    token: Option<String>,
    tag: Option<String>,
    host: Option<String>,
    parser: Option<Parser>,
    timeout: Option<Duration>,
}

impl LogzioBuilder {
    pub fn new() -> Self {
        LogzioBuilder {
            tag: None,
            token: None,
            host: None,
            parser: None,
            timeout: None,
        }
    }

    pub fn with_tag(mut self, tag: String) -> Self {
        self.tag = Some(tag);
        self
    }

    pub fn with_host(mut self, host: String) -> Self {
        self.host = Some(host);
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

    pub fn build(self) -> Result<Logzio> {
        match self {
            Self {
                tag: Some(tag),
                token: Some(token),
                host: Some(host),
                parser: Some(parser),
                timeout: _,
            } => {
                let client = match self.timeout {
                    Some(duration) => Client::builder().timeout(duration).build()?,
                    None => Client::builder().build()?,
                };

                Ok(Logzio {
                    url: format!("http://{}:8070/?token={}&type={}", host, token, tag),
                    parser,
                    client,
                })
            }
            Self { token: None, .. } => Err(Error::msg("Token Required")),
            Self { tag: None, .. } => Err(Error::msg("Tag Required")),
            Self { parser: None, .. } => Err(Error::msg("Parser Required")),
            Self { host: None, .. } => Err(Error::msg("Host Required")),
        }
    }
}
