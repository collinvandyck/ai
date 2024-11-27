use anyhow::{Context, Result};
use base64::{prelude::BASE64_STANDARD, Engine};
use eventsource_stream::Eventsource;
use futures::StreamExt;
use reqwest::{Method, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
    str::FromStr,
    sync::Arc,
    time::Duration,
};

pub struct Client {
    key: String,
    endpoint: url::Url,
    model: String,
    version: String,
    max_tokens: u32,
    client: reqwest::Client,
}

const SONNET: &str = "claude-3-5-sonnet-latest";
const HAIKU: &str = "claude-3-5-haiku-latest";

impl Client {
    pub fn new(key: String) -> Result<Self> {
        let endpoint = url::Url::parse("https://api.anthropic.com/").context("parse endpoint")?;
        let model = String::from(SONNET);
        let version = String::from("2023-06-01");
        let max_tokens = 4096;
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
            stream: false,
            messages: vec![Message {
                role: String::from("user"),
                content: vec![
                    Content::image_path(&image).await.context("image_path")?,
                    Content::text("what is in this image?"),
                ],
            }],
            ..Default::default()
        };
        let req = self
            .new_http_req(method, url)
            .json(&body)
            .build()
            .context("build request")?;
        let resp = self.client.execute(req).await.context("exec req")?;
        let code = resp.status();
        let text = resp.text().await.context("resp text")?;
        let resp = match serde_json::from_str(&text).context("parse json") {
            Ok(v) => v,
            Err(err) => {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                    let text = serde_json::to_string_pretty(&val).context("pretty json error")?;
                    tracing::error!("Failed to parse:\n{text}");
                } else {
                    tracing::error!("Failed to parse:\n{text}");
                }
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
            stream: false,
            messages: vec![Message {
                role: String::from("user"),
                content: vec![Content::text(&msg)],
            }],
            ..Default::default()
        };
        let req = self
            .new_http_req(method, url)
            .json(&body)
            .build()
            .context("build request")?;
        let resp = self.client.execute(req).await.context("exec req")?;
        let text = resp.text().await.context("resp text")?;
        let resp = match serde_json::from_str(&text).context("parse json") {
            Ok(v) => v,
            Err(err) => {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                    let text = serde_json::to_string_pretty(&val).context("pretty json error")?;
                    tracing::error!("Failed to parse:\n{text}");
                }
                tracing::error!("Failed to parse text:\n{text}");
                return Err(err);
            }
        };
        Ok(resp)
    }

    pub async fn stream_speak(&self, msg: &str) -> Result<()> {
        let method = reqwest::Method::POST;
        let url = self.endpoint.join("/v1/messages").context("build url")?;
        let body = MessagesRequest {
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            stream: true,
            messages: vec![Message {
                role: String::from("user"),
                content: vec![Content::text(&msg)],
            }],
            system: Some(String::from(
                "you are a helpful, wise modern day carl sagan.",
            )),
            ..Default::default()
        };
        let bjson = serde_json::to_string_pretty(&body).context("string pretty")?;
        tracing::info!("Sending request:\n{bjson}");
        let req = self
            .new_http_req(method, url)
            .json(&body)
            .build()
            .context("build request")?;
        let mut stream = self
            .client
            .execute(req)
            .await
            .context("exec req")?
            .bytes_stream()
            .eventsource();
        while let Some(event) = stream.next().await {
            match event {
                Ok(event) => {
                    let data = event.data;
                    match serde_json::from_str::<StreamEvent>(&data) {
                        Ok(event) => match event {
                            StreamEvent::MessageStart {
                                message: MessagesResponse { content, .. },
                            } => {
                                if !content.is_empty() {
                                    anyhow::bail!("message start was not empty: {content:#?}");
                                }
                            }
                            StreamEvent::StartBlock { index, content } => {
                                print!("{content}");
                            }
                            StreamEvent::BlockDelta { index, delta } => {
                                print!("{delta}");
                            }
                            StreamEvent::BlockStop { index } => {}
                            StreamEvent::MessageDelta { delta } => {}
                            StreamEvent::MessageStop => {
                                println!();
                            }
                            StreamEvent::Ping => {}
                        },
                        Err(err) => {
                            tracing::error!("failed to unmarshal\n{data}");
                            anyhow::bail!(err);
                        }
                    }
                    std::io::stdout().flush().context("flush stdout")?;
                }
                Err(err) => {
                    anyhow::bail!("event stream err: {err}");
                }
            }
        }
        Ok(())
    }

    fn new_http_req(&self, method: reqwest::Method, url: impl reqwest::IntoUrl) -> RequestBuilder {
        self.client
            .request(method, url)
            .header("x-api-key", &self.key)
            .header("anthropic-version", &self.version)
            .header("content-type", "application/json")
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum StreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: MessagesResponse },
    #[serde(rename = "content_block_start")]
    StartBlock {
        index: usize,
        #[serde(rename = "content_block")]
        content: Content,
    },
    #[serde(rename = "content_block_delta")]
    BlockDelta { index: usize, delta: Content },
    #[serde(rename = "content_block_stop")]
    BlockStop { index: usize },
    #[serde(rename = "message_delta")]
    MessageDelta { delta: MessagesResponse },
    #[serde(rename = "message_stop")]
    MessageStop,
    #[serde(rename = "ping")]
    Ping,
}

// Anthropic response for all of its apis
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Response {
    #[serde(rename = "message")]
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

#[derive(Debug, Serialize, Default)]
struct MessagesRequest {
    model: String,
    max_tokens: u32,
    stream: bool,
    system: Option<String>,
    messages: Vec<Message>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct MessagesResponse {
    content: Vec<Content>,
    id: String,
    model: String,
    role: String,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: Option<Usage>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum Content {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "text_delta")]
    TextDelta { text: String },
    #[serde(rename = "image")]
    Image { source: ImageSource },
}

impl std::fmt::Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Content::Text { text } => write!(f, "{text}"),
            Content::TextDelta { text } => write!(f, "{text}"),
            Content::Image {
                source: ImageSource {
                    media_type, data, ..
                },
            } => write!(f, "[{media_type} ({} bytes)]", data.len()),
        }
    }
}

impl Content {
    fn text(s: impl ToString) -> Self {
        Content::Text {
            text: s.to_string(),
        }
    }

    async fn image_path(p: impl AsRef<Path>) -> Result<Self> {
        let mime = mime_guess::from_path(&p)
            .first()
            .context("no mime type from filename")?;
        let bs = tokio::fs::read(&p).await.context("read file")?;
        let mut data = String::new();
        BASE64_STANDARD.encode_string(&bs, &mut data);
        Ok(Self::Image {
            source: ImageSource {
                typ: String::from("base64"),
                media_type: mime.to_string(),
                data: data.to_string(),
            },
        })
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImageSource {
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

    #[test]
    fn serde_err_resp2() {
        let json = r#"
            {
              "content": [
                {
                  "text": "Use reqwest with serde for JSON serialization",
                  "type": "text"
                }
              ],
              "id": "msg_01BhheLXdCtJUbMsZ3enae5i",
              "model": "claude-3-5-sonnet-20241022",
              "role": "assistant",
              "stop_reason": "end_turn",
              "stop_sequence": null,
              "type": "message",
              "usage": {
                "input_tokens": 19,
                "output_tokens": 40
              }
            }
        "#;
        let c: Response = serde_json::from_str(json).unwrap();
    }
}
