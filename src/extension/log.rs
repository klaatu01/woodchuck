use super::{base_url,ExtensionId, EXTENSION_ID_HEADER};
use crate::{LogQueue, LogDest};
use reqwest::blocking::Client;
use serde::Deserialize;
use warp::{path, serve, Filter, Reply};
use std::env;

const MAX_ITEMS_DEFAULT: u32 = 1000;
const MAX_BYTES_DEFAULT: u32 = 262144; //This needs to be configurable with an envar;
const TIMEOUT_DEFAULT: u32 = 5000; //This shouldnt need to be this high anymore as we are accepting logs async now;
const PORT_DEFAULT: u16 = 1060;

pub struct LogSubscriptionConfig {
    port: u16, 
    max_items: u32,
    max_bytes: u32,
    timeout: u32
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
                    \"URI\":\"http://sandbox:{}\"
                }},
                \"types\": 
                [
                    \"platform\", 
                    \"function\"
                ],
                \"buffering\": 
                {{
                     \"maxItems\": {},
                     \"maxBytes\": {},
                     \"timeoutMs\": {}
                 }}
             }}",
             config.port, config.max_items, config.max_bytes, config.timeout)
        .as_str())
        .unwrap()
}

pub fn subscribe(config: &LogSubscriptionConfig,client: &Client, ext_id: &ExtensionId) {
    let body = log_subscription_request(&config);
    let url = format!("{}/2020-08-15/logs", base_url().unwrap());
    client
        .put(&url)
        .header(EXTENSION_ID_HEADER, ext_id)
        .json(&body)
        .send()
        .unwrap();
}

#[derive(Default, Debug, Deserialize, Clone)]
pub struct CloudWatchLog {
    pub time: String,
    pub r#type: String,
    pub record: serde_json::Value,
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


pub fn start_log_consumer(log_queue: LogQueue, log_dest: LogDest){
    async fn run(queue: LogQueue, dest: LogDest) {
        loop {
            consume(&queue, &dest).await;
            std::thread::sleep(std::time::Duration::from_secs(10));
        }
    };
    tokio::spawn(run(log_queue, log_dest));
}

pub async fn consume(queue: &LogQueue, dest:&LogDest) {
    let length = queue.read().await.len();
    match length
    {
        0 => (),
        _ => {
            let split = queue.write().await.split_off(0).clone();
            match dest.read().await.handle_logs(split.clone()) {
                Ok(_) => (),
                Err(e) => {
                    println!("ERROR {}", e.to_string());
                    println!("failed to send {}, appending back to queue",split.len());
                    queue.write().await.extend(split);
                    ()
                },
            }
        }
    }
}

async fn handle_log(
    logs: Vec<CloudWatchLog>,
    log_queue: LogQueue,
) -> Result<impl Reply, std::convert::Infallible> {
    log_queue.write().await.append(&mut logs.clone());
    Ok(warp::reply())
}

#[cfg(test)]
mod tests {

    use super::CloudWatchLog;
    use super::LogQueue;
    use crate::{Arc, LogDest, RwLock};
    use super::consume;
    #[tokio::test]
    async fn consume_log() {
        //Arrange
        let queue: LogQueue = Arc::new(RwLock::new(Vec::new()));
        let dest: LogDest = crate::destination::get_test_destination().unwrap();
        queue.write().await.push(
            CloudWatchLog { 
                record:
            serde_json::Value::String("2020-11-18T23:52:30.128Z\t6e48723a-1596-4313-a9af-e4da9214d637\tINFO\tHello World\n".to_string())
                , ..Default::default()
            }
        );
        //Act
        consume(&queue,&dest).await;

        //Assert
        assert_eq!(queue.read().await.len(), 0);
    }
}
