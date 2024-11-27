use tracing::info;

#[tokio::main]
async fn main() {
    ai::tracing::init();
    info!("Hello.");
}
