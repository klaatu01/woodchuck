use crate::handler::{LogHandler, LogHandlerResponse};
use crate::models::Log;
use anyhow::Result;
use async_trait::async_trait;
use byte_chunk::SafeByteChunkedMut;
use rusoto_core::Region;
use rusoto_firehose::{KinesisFirehose, KinesisFirehoseClient, PutRecordInput, Record};

use serde::Serialize;

pub struct Firehose {
    delivery_stream_name: String,
    metadata: serde_json::Value,
    client: KinesisFirehoseClient,
}

#[derive(Debug, Serialize, Clone)]
pub struct FirehoseData {
    metadata: serde_json::Value,
    logs: Vec<String>,
}

impl Firehose {
    pub fn new(delivery_stream_name: String, metadata: serde_json::Value) -> Self {
        Firehose {
            delivery_stream_name,
            metadata,
            client: KinesisFirehoseClient::new(Region::default()),
        }
    }

    async fn send_logs(&self, logs: &[Log]) -> Result<()> {
        let collection: Vec<String> = logs.iter().map(|x| x.to_string()).collect::<Vec<String>>();

        let firehose_log = FirehoseData {
            metadata: self.metadata.clone(),
            logs: collection,
        };

        let data = serde_json::to_string(&firehose_log)?;

        let encoded_data = base64::encode(data);

        let record = Record { data: encoded_data.into() };

        let input = PutRecordInput {
            delivery_stream_name: self.delivery_stream_name.clone(),
            record,
        };

        let _ = self.client.put_record(input).await?;

        Ok(())
    }
}

#[async_trait]
impl LogHandler for Firehose {
    async fn handle_logs(&self, logs: Vec<Log>) -> LogHandlerResponse {
        let mut local_logs = logs.to_owned();
        let chunks = local_logs.byte_chunks_safe_mut(900000);

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

        match failed_to_send_logs.len() {
            0 => Ok(()),
            _ => Err(failed_to_send_logs.into()),
        }
    }
}
