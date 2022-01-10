use anyhow::ensure;
use anyhow::Result;
use reqwest::Client;
use std::collections::HashMap;

pub type ExtensionId = String;

pub mod logs_api;
pub mod runtime;

pub const EXTENSION_NAME: &str = "woodchuck";
pub const EXTENSION_HEADER_NAME: &str = "Lambda-Extension-Name";
pub const EXTENSION_ID_HEADER: &str = "Lambda-Extension-Identifier";

cfg_if::cfg_if! {
    if #[cfg(feature = "arm64")] {
        const TARGET_ARCHITECTURE: Option<&str> = Some("arm64");
    }
    else if #[cfg(feature = "x86_64")] {
        const TARGET_ARCHITECTURE: Option<&str> = Some("x86_64");
    }
    else {
        const TARGET_ARCHITECTURE: Option<&str> = None;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "loggly")] {
        const TARGET_DESTINATION: Option<&str> = Some("loggly");
    }
    else if #[cfg(feature = "logzio")] {
        const TARGET_DESTINATION: Option<&str> = Some("logzio");
    }
    else if #[cfg(feature = "firehose")] {
        const TARGET_DESTINATION: Option<&str> = Some("firehose");
    }
    else {
        const TARGET_DESTINATION: Option<&str> = None;
    }
}

pub fn get_extension_name() -> String {
    let name = match (TARGET_ARCHITECTURE, TARGET_DESTINATION) {
        (Some(arch), Some(dest)) => format!("{}_{}_{}", EXTENSION_NAME, dest, arch),
        (_, _) => EXTENSION_NAME.to_string(),
    };
    cfg_if::cfg_if! {
        if #[cfg(feature = "dev")] {
            name + "_dev"
        } else {
            name
        }
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
        .header(EXTENSION_HEADER_NAME, get_extension_name())
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
