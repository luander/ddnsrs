use clap::Parser;
use ddnsrs::{cf::CFClient, cli::Cli, ip::*};
use std::{env, error::Error};
use tokio::join;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let start = std::time::Instant::now();
    setup_tracing();
    let args = Cli::parse();
    let hostname = &args.hostname;
    let cf_api_key = env::var("CLOUDFLARE_API_KEY").expect("CLOUDFLARE_API_KEY is not set");
    let cf_client = CFClient::new(cf_api_key);
    let (pip, dns_record) = join!(get_pip(), cf_client.dns_record(hostname));
    let pip = pip?;
    let dns_record = dns_record?;
    info!(
        "{} = {}; public IP = {}",
        hostname, &dns_record["content"], &pip
    );
    if !pip.eq(&dns_record["content"]) {
        cf_client
            .update_record(&hostname, &dns_record, &pip)
            .await?;
    }
    let duration = start.elapsed();
    info!("finished in {}", duration.as_secs_f64());
    Ok(())
}

fn setup_tracing() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}
