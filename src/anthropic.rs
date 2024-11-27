use std::{str::FromStr, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use reqwest::{Method, RequestBuilder};
use serde::{Deserialize, Serialize};

pub struct Client {
    key: String,
    endpoint: url::Url,
    model: String,
    version: String,
    max_tokens: u32,
    client: reqwest::Client,
}

impl Client {
    pub fn new(key: String) -> Result<Self> {
        let endpoint = url::Url::parse("https://api.anthropic.com/").context("parse endpoint")?;
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

    pub async fn speak(&self, msg: &str) -> Result<MessagesResponse> {
        let method = reqwest::Method::POST;
        let url = self.endpoint.join("/v1/messages").context("build url")?;
        let body = MessagesRequest {
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            messages: vec![Message {
                role: String::from("user"),
                content: msg.to_string(),
            }],
        };
        let req = self
            .new_request(method, url)
            .json(&body)
            .build()
            .context("build request")?;
        let resp = self.client.execute(req).await.context("exec req")?;
        let resp = resp.json::<MessagesResponse>().await.context("resp json")?;
        Ok(resp)
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
pub struct MessagesResponse {
    content: Vec<Message>,
    id: String,
    model: String,
    #[serde(default)]
    role: Option<String>,
    stop_reason: String,
    #[serde(default)]
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
