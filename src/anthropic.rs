use std::{sync::Arc, time::Duration};

use anyhow::{Context, Result};
use reqwest::RequestBuilder;
use serde::{Deserialize, Serialize};

pub struct Client {
    key: String,
    endpoint: String,
    model: Arc<String>,
    version: String,
    max_tokens: u32,
    client: reqwest::Client,
}

impl Client {
    pub fn new(key: String) -> Result<Self> {
        let endpoint = String::from("https://api.anthropic.com");
        let model = Arc::new(String::from("claude-3-5-sonnet-20241022"));
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

    async fn speak(&self, msg: &str) -> Result<MessagesResponse> {
        todo!()
    }

    fn new_request(&self, method: reqwest::Method, url: impl reqwest::IntoUrl) -> RequestBuilder {
        self.client
            .request(method, url)
            .header("x-api-key", &self.key)
            .header("anthropic-version", &self.version)
            .header("content-type", "application/json")
    }
}

#[derive(Debug, Serialize)]
struct MessagesRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Debug, Deserialize)]
struct MessagesResponse {
    content: Vec<Message>,
    id: String,
    model: String,
    role: String,
    stop_reason: String,
    stop_sequence: Option<String>,
    #[serde(rename = "type")]
    kind: String,
    usage: Usage,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
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

#[derive(Debug, Deserialize)]
pub struct Usage {
    input_tokens: u64,
    output_tokens: u64,
}
