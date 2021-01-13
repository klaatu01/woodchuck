#[macro_use]
extern crate serde;
extern crate serde_json;

mod extension;
mod handler;
mod models;
mod parser;

use anyhow::Result;
use extension::{logs_api, runtime};
use reqwest::blocking::Client;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    debug!("Building Client");
    let client = Client::builder().timeout(None).build()?;
    let log_queue = models::new_log_queue();
    let log_dest = handler::get_default()?;
    let log_config = logs_api::LogSubscriptionConfig::default();

    debug!("Registering Extension...");
    let ext_id = extension::register_extension(&client)?;
    debug!("Registered.");

    logs_api::start_log_server(&log_config, log_queue.clone()); //We need to start running our server before we register as a log extension
    logs_api::subscribe(&log_config, &client, &ext_id);
    let response = runtime::run(&client, ext_id, log_queue, log_dest).await;
    response
}
