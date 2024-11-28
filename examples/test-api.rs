use std::{env, error::Error, io::Write};

use anyhow::Context;
use clap::Parser;
use tokio::io::{AsyncBufReadExt, BufReader};
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
    StreamSpeak,
    Repl,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    ai::tracing::init();
    let args = Args::parse();
    let key = env::var("ANTHROPIC_API_KEY").context("no api key")?;
    let client = ai::anthropic::Client::new(key).context("new client")?;
    match &args.cmd {
        Command::Speak => {
            let resp = client
                .speak("any one sentence tips for writing an anthropic rust client?")
                .await
                .context("speak")?;
            info!("Response:\n{resp:#?}");
        }
        Command::Image => {
            let resp = client.explain_image("images/collin.jpeg").await.context("explain_image")?;
            info!("Response:\n{resp:#?}");
        }
        Command::StreamSpeak => {
            client.stream_speak("explain HDR").await?;
        }
        Command::Repl => {
            let mut input = BufReader::new(tokio::io::stdin());
            loop {
                print!("> ");
                std::io::stdout().flush()?;
                let mut buf = String::new();
                if input.read_line(&mut buf).await? == 0 {
                    println!();
                    break;
                }
                println!();
                let buf = buf.trim();
                if !buf.is_empty() {
                    client.stream_speak(buf).await?;
                    println!();
                }
            }
        }
    };
    Ok(())
}
