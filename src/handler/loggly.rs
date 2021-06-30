use crate::handler::LogHandler;
use crate::models::Log;
use anyhow::{ensure, Error, Result};
use async_trait::async_trait;
use byte_chunk::SafeByteChunkedMut;
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Loggly {
    url: String,
    client: Client,
}

impl Loggly {
    pub fn builder() -> LogglyBuilder {
        LogglyBuilder::new()
    }

    async fn send_logs(&self, logs: &[Log]) -> Result<()> {
        let payload: String = logs
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join("\n");

        log::debug!(
            "Sending {} logs, payload length: {}",
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
    async fn handle_logs(&self, logs: Vec<Log>) -> (Result<()>, Vec<Log>) {
        let mut local_logs = logs.to_owned();
        let chunks = local_logs.byte_chunks_safe_mut(4900000); //give ourselves 100kb overhead to be safe.

        let mut failed_to_send_logs = Vec::<Log>::new();

        for (index, chunk) in chunks.enumerate() {
            let rslt = self.send_logs(chunk).await;
            match rslt {
                Err(e) => {
                    log::debug!("Failed sending Chunk {} with {} items.", index, chunk.len());
                    failed_to_send_logs.extend_from_slice(chunk);
                    log::error!("{}", e)
                }
                _ => log::debug!("Sent Chunk {} with {} items.", index, chunk.len()),
            }
        }

        (Ok(()), failed_to_send_logs)
    }
}

pub struct LogglyBuilder {
    token: Option<String>,
    tag: Option<String>,
    timeout: Option<Duration>,
}

impl LogglyBuilder {
    pub fn new() -> Self {
        LogglyBuilder {
            tag: None,
            token: None,
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
                timeout: _,
            } => {
                let client = match self.timeout {
                    Some(duration) => Client::builder().timeout(duration).build()?,
                    None => Client::builder().build()?,
                };

                Ok(Loggly {
                    url: format!("http://logs-01.loggly.com/bulk/{}/tag/{}/", token, tag),
                    client,
                })
            }
            Self { token: None, .. } => Err(Error::msg("Token Required")),
            Self { tag: None, .. } => Err(Error::msg("Tag Required")),
        }
    }
}
