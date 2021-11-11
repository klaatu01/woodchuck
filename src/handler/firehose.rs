use crate::handler::LogHandler;
use crate::models::RawCloudWatchLog;
use crate::parser::Parser;
use anyhow::Result;
use async_trait::async_trait;
use rusoto_core::Region;
use rusoto_firehose::{KinesisFirehose, KinesisFirehoseClient, PutRecordInput, Record};

use serde::Serialize;

pub struct Firehose {
    delivery_stream_name: String,
    tag: String,
    parser: Parser,
    client: KinesisFirehoseClient,
}

#[derive(Debug, Serialize, Clone)]
pub struct FirehoseData {
    tag: String,
    logs: Vec<String>,
}

impl Firehose {
    pub fn new(delivery_stream_name: String, tag: String) -> Self {
        Firehose {
            delivery_stream_name,
            tag,
            parser: Parser,
            client: KinesisFirehoseClient::new(Region::default()),
        }
    }
}

#[async_trait]
impl LogHandler for Firehose {
    async fn handle_logs(&self, cloudwatch_logs: Vec<RawCloudWatchLog>) -> Result<()> {
        let logs: Vec<String> = self
            .parser
            .parse(cloudwatch_logs)
            .iter()
            .map(|log| log.to_string())
            .collect::<Vec<String>>();

        let firehose_log = FirehoseData {
            tag: self.tag.clone(),
            logs,
        };

        let data = serde_json::to_string(&firehose_log)?;

        let record = Record { data: data.into() };

        let input = PutRecordInput {
            delivery_stream_name: self.delivery_stream_name.clone(),
            record,
        };

        let _response = self.client.put_record(input).await?;

        Ok(())
    }
}
