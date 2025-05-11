use anyhow::Result;
use clap::Parser;
use lnaddrd::{config::Config, serve};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::parse();
    serve(&config).await
}
