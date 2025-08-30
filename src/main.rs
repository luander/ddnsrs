use clap::Parser;
use ddnsrs::{cache::*, cf::CFClient, cli::Cli, ip::*};
use std::{env, error::Error};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{filter::EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let start = std::time::Instant::now();
    setup_tracing();
    let args = Cli::parse();
    let hostname = &args.hostname;
    let cf_api_key = env::var("CLOUDFLARE_API_KEY").expect("CLOUDFLARE_API_KEY is not set");

    let pip = get_pip().await?;
    let cached_pip = read_cached_ip()?;

    if Some(&pip) == cached_pip.as_ref() {
        info!(
            "Actual IP {} == Cached IP {}. Exiting.",
            pip,
            cached_pip.unwrap_or_default()
        );
        print_elapsed(start);
        return Ok(());
    }

    let mut cf_client = CFClient::new(cf_api_key);
    let dns_record = cf_client.dns_record(hostname).await?;
    info!(
        "{} = {}; public IP = {}",
        hostname, &dns_record["content"], &pip
    );
    if !pip.eq(&dns_record["content"]) {
        cf_client.update_record(hostname, &dns_record, &pip).await?;
    }
    write_cached_ip(&pip)?;
    print_elapsed(start);
    Ok(())
}

fn print_elapsed(start: std::time::Instant) {
    let duration = start.elapsed();
    info!("finished in {} seconds", duration.as_secs_f64());
}

fn setup_tracing() {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    let subscriber = FmtSubscriber::builder().with_env_filter(filter).finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}
