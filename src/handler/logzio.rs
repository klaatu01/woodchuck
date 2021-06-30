use crate::handler::LogHandler;
use crate::models::Log;
use anyhow::{ensure, Error, Result};
use async_trait::async_trait;
use byte_chunk::SafeByteChunkedMut;
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Logzio {
    url: String,
    client: Client,
}

impl Logzio {
    pub fn builder() -> LogzioBuilder {
        LogzioBuilder::new()
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
impl LogHandler for Logzio {
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

pub struct LogzioBuilder {
    token: Option<String>,
    host: Option<String>,
    timeout: Option<Duration>,
}

impl LogzioBuilder {
    pub fn new() -> Self {
        LogzioBuilder {
            token: None,
            host: None,
            timeout: None,
        }
    }

    pub fn with_host(mut self, host: String) -> Self {
        self.host = Some(host);
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

    pub fn build(self) -> Result<Logzio> {
        match self {
            Self {
                token: Some(token),
                host: Some(host),
                timeout: _,
            } => {
                let client = match self.timeout {
                    Some(duration) => Client::builder().timeout(duration).build()?,
                    None => Client::builder().build()?,
                };

                Ok(Logzio {
                    url: format!("http://{}:8070/?token={}&type=http-bulk", host, token),
                    client,
                })
            }
            Self { token: None, .. } => Err(Error::msg("Token Required")),
            Self { host: None, .. } => Err(Error::msg("Host Required")),
        }
    }
}
