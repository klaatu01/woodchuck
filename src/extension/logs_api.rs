use super::{base_url,ExtensionId, EXTENSION_ID_HEADER};
use crate::models::{LogQueue, RawCloudWatchLog};
use crate::handler::{Handler, FailedToSendLogsError};
use reqwest::Client;
use warp::{path, serve, Filter, Reply};
use std::{env, thread::sleep, time::Duration};
use crate::parser::parse;

const MAX_ITEMS_DEFAULT: u32 = 1000;
const MAX_BYTES_DEFAULT: u32 = 262144;
const TIMEOUT_DEFAULT: u32 = 2500; 

const PORT_DEFAULT: u16 = 1060;
const HOST_DEFAULT: &str = "sandbox";

pub struct LogSubscriptionConfig {
    port: u16, 
    max_items: u32,
    max_bytes: u32,
    timeout: u32,
    host: String,
}

impl Default for LogSubscriptionConfig {
    fn default() -> Self {
        LogSubscriptionConfig {
            max_items: match env::var("WOODCHUCK_MAX_ITEMS") {
                Ok(var) => var.parse().unwrap(),
                Err(_) => MAX_ITEMS_DEFAULT,
            },
            max_bytes: match env::var("WOODCHUCK_MAX_BYTES") {
                Ok(var) => var.parse().unwrap(),
                Err(_) => MAX_BYTES_DEFAULT,
            },
            timeout: match env::var("WOODCHUCK_TIMEOUT") {
                Ok(var) => var.parse().unwrap(),
                Err(_) => TIMEOUT_DEFAULT,
            },
            port: match env::var("WOODCHUCK_PORT") {
                Ok(var) => var.parse().unwrap(),
                Err(_) => PORT_DEFAULT,
            },
            host: match env::var("WOODCHUCK_HOST") {
                Ok(var) => var.parse().unwrap(),
                Err(_) => HOST_DEFAULT.to_string(),
            },
        }
    }
}

fn log_subscription_request(config: &LogSubscriptionConfig) -> serde_json::Value {
    serde_json::from_str(
        format!(
            "{{ 
                \"destination\": 
                {{ 
                    \"protocol\": \"HTTP\", 
                    \"URI\":\"http://{}:{}\"
                }},
                \"types\": 
                [
                    \"function\"
                ],
                \"buffering\": 
                {{
                     \"maxItems\": {},
                     \"maxBytes\": {},
                     \"timeoutMs\": {}
                 }}
             }}",
             config.host, config.port, config.max_items, config.max_bytes, config.timeout)
        .as_str())
        .unwrap()
}

pub async fn subscribe(config: &LogSubscriptionConfig,client: &Client, ext_id: &ExtensionId) {
    let body = log_subscription_request(&config);
    let url = format!("{}/2020-08-15/logs", base_url().unwrap());
    client
        .put(&url)
        .header(EXTENSION_ID_HEADER, ext_id)
        .json(&body)
        .send().await
        .unwrap();
}

pub fn start_log_server(config: &LogSubscriptionConfig, log_queue: LogQueue) {
    async fn run(port: u16, log_queue: LogQueue) {
        let routes = path::end()
            .and(warp::post())
            .and(warp::body::json())
            .and(with_log_queue(log_queue))
            .and_then(handle_log);
        serve(routes).run(([0, 0, 0, 0], port)).await;
    }
    tokio::spawn(run(config.port, log_queue));
}

fn with_log_queue(
    log_queue: LogQueue,
) -> impl Filter<Extract = (LogQueue,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || log_queue.clone())
}

pub async fn consume_retry(queue: &LogQueue, dest:&Handler, attempts: u64, sleep_ms: u64) {
    for _ in 0..attempts {
        if consume(queue, dest).await == true {
            break;
        }
        sleep(Duration::from_millis(sleep_ms));
    }
}

pub async fn consume(queue: &LogQueue, dest:&Handler) -> bool {
    let length = queue.read().await.len();
    match length
    {
        0 => false,
        _ => {
            let split = queue.write().await.split_off(0);
            match dest.read().await.handle_logs(split).await {
                Ok(_) => true,
                Err(FailedToSendLogsError{logs}) => {
                    println!("failed to send {}, appending back to queue",logs.len());
                    queue.write().await.extend(logs);
                    false
                },
            }
        }
    }
}

async fn handle_log(
    logs: Vec<RawCloudWatchLog>,
    log_queue: LogQueue,
) -> Result<impl Reply, std::convert::Infallible> {
    log::debug!("Adding {} logs", logs.len());
    log_queue.write().await.append(&mut parse(logs.clone()));
    log::debug!("Added {} logs", logs.len());
    Ok(warp::reply())
}

#[cfg(test)]
mod tests {
    use super::RawCloudWatchLog;
    use crate::models::new_log_queue;
    use super::{consume, handle_log};
    #[tokio::test]
    async fn consume_log() {
        //Arrange
        let queue = new_log_queue();
        let dest = crate::handler::get_test_destination().unwrap();
        let rslt = handle_log(
            vec![RawCloudWatchLog { 
                record:
            serde_json::Value::String("2020-11-18T23:52:30.128Z\t6e48723a-1596-4313-a9af-e4da9214d637\tINFO\tHello World\n".to_string())
                , ..Default::default()
            }], queue.clone()
        ).await;

        match rslt {
            Ok(_) => assert!(true),
            Err(e) => assert!(false, e),
        };
        //Act
        consume(&queue,&dest).await;

        //Assert
        assert_eq!(queue.read().await.len(), 0);
    }
}
