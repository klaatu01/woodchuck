#[macro_use]
extern crate serde;
extern crate log;
extern crate serde_json;

mod extension;
mod handler;
mod models;
mod parser;

use anyhow::Result;
use extension::{logs_api, runtime};
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    log::debug!("Building {} Client", extension::EXTENSION_NAME);
    let client = Client::builder().build()?;
    log::debug!("Built Client");
    let log_queue = models::new_log_queue();
    let log_dest = handler::get_default()?;
    let log_config = logs_api::LogSubscriptionConfig::default();

    log::debug!("Registering Extension...");
    let ext_id = extension::register_extension(&client).await?;
    log::debug!("Registered.");

    log::debug!("Starting Log Server...");
    logs_api::start_log_server(&log_config, log_queue.clone()); //We need to start running our server before we register as a log extension
    log::debug!("Started Log Server.");
    log::debug!("Registering Log Server");
    logs_api::subscribe(&log_config, &client, &ext_id).await;
    log::debug!("Registered.");
    log::debug!("Starting Runtime Consumer...");
    let response = runtime::run(&client, ext_id, log_queue, log_dest).await;
    response
}
