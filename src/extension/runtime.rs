use super::{base_url, log, ExtensionId, EXTENSION_ID_HEADER};
use crate::destination::Destination;
use crate::models::LogQueue;
use anyhow::Result;
use reqwest::blocking::Client;

pub async fn run(
    client: &Client,
    ext_id: ExtensionId,
    log_queue: LogQueue,
    log_dest: Destination,
) -> Result<()> {
    loop {
        match next_event(&client, &ext_id) {
            Ok(evt) => match evt {
                NextEventResponse::Invoke { request_id, .. } => {
                    println!("{}", request_id);
                    log::consume(&log_queue, &log_dest).await;
                }
                NextEventResponse::Shutdown {
                    shutdown_reason, ..
                } => {
                    println!("Exiting: {}", shutdown_reason);
                    log::consume(&log_queue, &log_dest).await;
                    return Ok(());
                }
            },
            Err(err) => {
                println!("Error: {:?}", err);
                log::consume(&log_queue, &log_dest).await;
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

fn next_event(client: &reqwest::blocking::Client, ext_id: &String) -> Result<NextEventResponse> {
    let url = format!("{}/2020-01-01/extension/event/next", base_url().unwrap());
    let response: reqwest::Result<NextEventResponse> = client
        .get(&url)
        .header(EXTENSION_ID_HEADER, ext_id)
        .send()?
        .json();

    match response {
        Ok(data) => Ok(data),
        Err(err) => {
            println!("{}", err.to_string());
            Err(anyhow::Error::msg(err.to_string()))
        }
    }
}
