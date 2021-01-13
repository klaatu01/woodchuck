use super::{base_url, logs_api, ExtensionId, EXTENSION_ID_HEADER};
use crate::handler::Handler;
use crate::models::LogQueue;
use anyhow::Result;
use reqwest::Client;

pub async fn run(
    client: &Client,
    ext_id: ExtensionId,
    log_queue: LogQueue,
    log_dest: Handler,
) -> Result<()> {
    loop {
        let event = next_event(&client, &ext_id).await;
        log::debug!("Next Event: {:?}", &event);
        match event {
            Ok(evt) => match evt {
                NextEventResponse::Invoke { request_id, .. } => {
                    log::debug!("Request Id: {:?}", request_id);
                    logs_api::consume(&log_queue, &log_dest).await;
                }
                NextEventResponse::Shutdown {
                    shutdown_reason, ..
                } => {
                    log::debug!("Exiting: {:?}", shutdown_reason);
                    logs_api::consume(&log_queue, &log_dest).await;
                    return Ok(());
                }
            },
            Err(err) => {
                log::debug!("Error: {:?}", err);
                logs_api::consume(&log_queue, &log_dest).await;
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Tracing {
    pub r#type: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE", tag = "eventType")]
enum NextEventResponse {
    #[serde(rename_all = "camelCase")]
    Invoke {
        deadline_ms: u64,
        request_id: String,
        invoked_function_arn: String,
        tracing: Tracing,
    },
    #[serde(rename_all = "camelCase")]
    Shutdown {
        shutdown_reason: String,
        deadline_ms: u64,
    },
}

async fn next_event(client: &reqwest::Client, ext_id: &String) -> Result<NextEventResponse> {
    let url = format!("{}/2020-01-01/extension/event/next", base_url().unwrap());
    let response: reqwest::Result<NextEventResponse> = client
        .get(&url)
        .header(EXTENSION_ID_HEADER, ext_id)
        .send()
        .await?
        .json()
        .await;

    match response {
        Ok(data) => Ok(data),
        Err(err) => {
            println!("{}", err.to_string());
            Err(anyhow::Error::msg(err.to_string()))
        }
    }
}
