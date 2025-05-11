use clap::Parser;
use lnaddrd::{api::serve, config::Config};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::parse();
    serve(&config).await
}
