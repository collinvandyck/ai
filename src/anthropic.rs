use std::time::Duration;

use anyhow::{Context, Result};
use serde::Deserialize;

pub struct Client {
    key: String,
    endpoint: String,
    model: String,
    version: String,
    max_tokens: u32,
    client: reqwest::Client,
}

impl Client {
    pub fn new(key: String) -> Result<Self> {
        let endpoint = String::from("https://api.anthropic.com");
        let model = String::from("claude-3-5-sonnet-20241022");
        let version = String::from("2023-06-01");
        let max_tokens = 1024;
        let client = reqwest::ClientBuilder::default()
            .timeout(Duration::from_secs(10))
            .build()
            .context("build http client")?;
        Ok(Self {
            key,
            endpoint,
            model,
            version,
            max_tokens,
            client,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct ServerError {
    #[serde(rename = "type")]
    kind: String,
    #[serde(rename = "error")]
    detail: ServerErrorDetail,
}

#[derive(Debug, Deserialize)]
pub struct ServerErrorDetail {
    #[serde(rename = "type")]
    kind: String,
    message: String,
}
