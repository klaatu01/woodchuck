#[macro_use]
extern crate serde;
extern crate serde_json;

use anyhow::Result;
use reqwest::blocking::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

mod extension;
use extension::{log, log::CloudWatchLog, runtime};

mod destination;
mod parser;

pub type LogQueue = Arc<RwLock<Vec<CloudWatchLog>>>;
pub type LogDest = Arc<RwLock<dyn destination::Destination + Sync + Send>>;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::builder().timeout(None).build()?;
    let log_queue: LogQueue = Arc::new(RwLock::new(Vec::new()));
    let log_dest: LogDest = destination::get_default()?;
    let log_config = log::LogSubscriptionConfig::default();

    let ext_id = extension::register_extension(&client)?;

    log::start_log_server(&log_config, log_queue.clone()); //We need to start running our server before we register as a log extension
    log::start_log_consumer(log_queue.clone(), log_dest.clone());
    log::subscribe(&log_config, &client, &ext_id);
    let response = runtime::run(&client, ext_id);

    //Flush logs on shutdown
    log::consume(&log_queue, &log_dest).await;

    response
}
