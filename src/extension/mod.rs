use anyhow::ensure;
use anyhow::Result;
use reqwest::Client;
use std::collections::HashMap;

pub type ExtensionId = String;

pub mod logs_api;
pub mod runtime;

pub const EXTENSION_HEADER_NAME: &str = "Lambda-Extension-Name";
pub const EXTENSION_ID_HEADER: &str = "Lambda-Extension-Identifier";

cfg_if::cfg_if! {
    if #[cfg(feature = "loggly")] {
        pub const EXTENSION_NAME: &str = "woodchuck_loggly";
    } else if #[cfg(feature = "logzio")] {
        pub const EXTENSION_NAME: &str = "woodchuck_logzio";
    } else {
        pub const EXTENSION_NAME: &str = "woodchuck";
    }
}

pub fn base_url() -> Option<String> {
    match std::env::var("AWS_LAMBDA_RUNTIME_API") {
        Ok(val) => Some(format!("http://{}", val)),
        Err(_) => None,
    }
}

pub async fn register_extension(client: &Client) -> Result<ExtensionId> {
    let mut map = HashMap::new();
    map.insert("events", vec!["INVOKE", "SHUTDOWN"]);
    let url = format!("{}/2020-01-01/extension/register", base_url().unwrap());
    let res = client
        .post(&url)
        .header(EXTENSION_HEADER_NAME, EXTENSION_NAME)
        .json(&map)
        .send()
        .await?;

    ensure!(
        res.status() == reqwest::StatusCode::OK,
        "Unable to register extension"
    );

    let ext_id = res.headers().get(EXTENSION_ID_HEADER).unwrap().to_str()?;

    Ok(ext_id.into())
}
