use std::{env, error::Error};

use anyhow::Context;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    ai::tracing::init();
    let key = env::var("ANTHROPIC_API_KEY").context("no api key")?;
    let client = ai::anthropic::Client::new(key).context("new client")?;
    let resp = client
        .speak("any one sentence tips for writing an anthropic rust client?")
        .await
        .context("speak")?;
    info!("Response: {resp:#?}");
    Ok(())
}
