use crate::handler::LogHandler;
use crate::models::RawCloudWatchLog;
use crate::parser::Parser;
use anyhow::{ensure, Error, Result};
use async_trait::async_trait;
use byte_chunk::SafeByteChunkedMut;
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
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

    async fn send_logs(&self, logs: &[String]) -> Result<()> {
        let payload: String = logs.join("\n");

        log::debug!(
            "Sending {} logs to Loggly, payload length: {}",
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

#[async_trait]
impl LogHandler for Loggly {
    async fn handle_logs(&self, cloudwatch_logs: Vec<RawCloudWatchLog>) -> Result<()> {
        let logs = self.parser.parse(cloudwatch_logs);

        let mut stringified_logs = logs
            .iter()
            .map(|log| log.to_string())
            .collect::<Vec<String>>();

        let chunks = stringified_logs.byte_chunks_safe_mut(4900000); //give ourselves 100kb overhead to be safe.

        for (index, chunk) in chunks.enumerate() {
            let rslt = self.send_logs(chunk).await?;
            match rslt {
                Error(e) => {
                    log::debug!("Failed sending Chunk {} with {} items.", index, chunk.len());
                    log::error!("{}", e)
                }
                _ => log::debug!("Sent Chunk {} with {} items.", index, chunk.len()),
            }
        }

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
            } => {
                let client = match self.timeout {
                    Some(duration) => Client::builder().timeout(duration).build()?,
                    None => Client::builder().build()?,
                };

                Ok(Loggly {
                    url: format!("http://logs-01.loggly.com/bulk/{}/tag/{}/", token, tag),
                    parser,
                    client,
                })
            }
            Self { token: None, .. } => Err(Error::msg("Token Required")),
            Self { tag: None, .. } => Err(Error::msg("Tag Required")),
            Self { parser: None, .. } => Err(Error::msg("Parser Required")),
        }
    }
}
