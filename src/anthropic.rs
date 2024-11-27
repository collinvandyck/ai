use serde::Deserialize;

pub struct Client {
    key: String,
    endpoint: String,
    model: String,
    version: String,
    max_tokens: u32,
}

impl Client {
    pub fn new(key: String) -> Self {
        let endpoint = String::from("https://api.anthropic.com");
        let model = String::from("claude-3-5-sonnet-20241022");
        let version = String::from("2023-06-01");
        let max_tokens = 1024;
        Self {
            key,
            endpoint,
            model,
            version,
            max_tokens,
        }
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
