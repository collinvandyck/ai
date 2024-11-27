use std::{path::Path, str::FromStr, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use reqwest::{Method, RequestBuilder};
use serde::{Deserialize, Serialize};
use tracing::info;

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

    pub async fn explain_image(&self, image: impl AsRef<Path>) -> Result<Response> {
        let method = reqwest::Method::POST;
        let url = self.endpoint.join("/v1/messages").context("build url")?;
        let body = MessagesRequest {
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            messages: vec![Message {
                role: String::from("user"),
                content: vec![
                    Content::image_path(&image).await.context("image_path")?,
                    Content::text("what is in this image?"),
                ],
            }],
        };
        let req = self
            .new_request(method, url)
            .json(&body)
            .build()
            .context("build request")?;
        let resp = self.client.execute(req).await.context("exec req")?;
        let code = resp.status();
        let text = resp.text().await.context("resp text")?;
        let resp = match serde_json::from_str(&text).context("parse json") {
            Ok(v) => v,
            Err(err) => {
                tracing::error!("Failed to parse text:\n{text}");
                return Err(err);
            }
        };
        Ok(resp)
    }

    pub async fn speak(&self, msg: &str) -> Result<Response> {
        let method = reqwest::Method::POST;
        let url = self.endpoint.join("/v1/messages").context("build url")?;
        let body = MessagesRequest {
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            messages: vec![Message {
                role: String::from("user"),
                content: vec![Content::text(&msg)],
            }],
        };
        let req = self
            .new_request(method, url)
            .json(&body)
            .build()
            .context("build request")?;
        let resp = self.client.execute(req).await.context("exec req")?;
        let text = resp.text().await.context("resp text")?;
        let resp = match serde_json::from_str(&text).context("parse json") {
            Ok(v) => v,
            Err(err) => {
                tracing::error!("Failed to parse text:\n{text}");
                return Err(err);
            }
        };
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

// Anthropic response for all of its apis
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Response {
    #[serde(rename = "messages")]
    Messages(MessagesResponse),
    #[serde(rename = "error")]
    Error { error: ServerError },
}

#[derive(Debug, Deserialize)]
pub struct ServerError {
    #[serde(rename = "type")]
    typ: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct MessagesRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Debug, Deserialize)]
pub struct MessagesResponse {
    content: Vec<Content>,
    id: String,
    model: String,
    role: String,
    stop_reason: String,
    stop_sequence: Option<String>,
    usage: Usage,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
enum Content {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { source: ImageSource },
}

impl Content {
    fn text(s: impl ToString) -> Self {
        Content::Text {
            text: s.to_string(),
        }
    }

    async fn image_path(p: impl AsRef<Path>) -> Result<Self> {
        let media_type = "";
        let data = "";
        Ok(Self::Image {
            source: ImageSource {
                typ: String::from("base64"),
                media_type: media_type.to_string(),
                data: data.to_string(),
            },
        })
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ImageSource {
    #[serde(rename = "type")]
    typ: String,
    media_type: String,
    data: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: Vec<Content>,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    input_tokens: u64,
    output_tokens: u64,
}

#[cfg(test)]
mod tests {
    use super::{Content, Response};

    #[test]
    fn serde_content() {
        let js = r#"{"type":"text", "text":"foobar"}"#;
        let c: Content = serde_json::from_str(js).unwrap();
        assert_eq!(
            c,
            Content::Text {
                text: String::from("foobar")
            }
        );
    }

    #[test]
    fn serde_error_resp() {
        let json = r#"{"type":"error","error":{"type":"invalid_request_error","message":"messages.0.content.0.image.source.media_type: Input should be 'image/jpeg', 'image/png', 'image/gif' or 'image/webp'"}}"#;
        let c: Response = serde_json::from_str(json).unwrap();
    }
}
