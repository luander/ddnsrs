use anyhow::Result;
use clap::Parser;
use ddnsrs::{cli::Cli, worker};

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing();

    let config = Cli::parse();

    tokio::spawn(worker::run(config)).await?
}

fn setup_tracing() {
    tracing_subscriber::fmt::init();
}
