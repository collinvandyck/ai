use std::{env, error::Error};

use anyhow::Context;
use clap::Parser;
use tracing::info;

#[derive(clap::Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    Speak,
    Image,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    ai::tracing::init();
    let args = Args::parse();
    let key = env::var("ANTHROPIC_API_KEY").context("no api key")?;
    let client = ai::anthropic::Client::new(key).context("new client")?;
    let resp = match &args.cmd {
        Command::Speak => client
            .speak("any one sentence tips for writing an anthropic rust client?")
            .await
            .context("speak")?,
        Command::Image => client
            .explain_image("foo.jpg")
            .await
            .context("explain_image")?,
    };
    info!("Response: {resp:#?}");
    Ok(())
}
