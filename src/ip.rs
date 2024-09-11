use anyhow::{Context, Result};
use std::net::ToSocketAddrs;
use tracing::{error, info, trace};

use crate::{cli::Cli, Message, MessageSender, ShutdownReceiver};

pub const PIP_PROVIDERS: [&str; 5] = [
    "https://ifconfig.me/ip",
    "https://icanhazip.com",
    "https://myip.dnsomatic.com",
    "https://myexternalip.com/raw",
    "https://api.ipify.org",
];

pub async fn run(config: Cli, sender: MessageSender, mut shutdown: ShutdownReceiver) -> Result<()> {
    let mut interval =
        tokio::time::interval(std::time::Duration::from_secs(config.interval as u64));
    loop {
        tokio::select! {
            _ = shutdown.recv() => return Ok(()),
            _ = interval.tick() => {
                let mut ips = vec![];
                for provider in PIP_PROVIDERS.iter() {
                    let now = std::time::Instant::now();
                    match get_pip(provider).await {
                        Ok(ip) => {
                            info!("{}: {}", provider, ip);
                            ips.push(ip);
                        },
                        Err(e) => error!("{}: {}", provider, e),
                    }
                    // TODO: add metric here
                    trace!("{} took: {:?}", provider, now.elapsed());
                }
                if let Some(ip) = ips.pop() {
                    let _ = sender.send(Message { ip });
                }
            }
        }
    }
}

pub async fn get_pip(url: &str) -> Result<String> {
    let public_ip = reqwest::get(url)
        .await
        .context(format!("Failed to execute GET on {}", url))?
        .text()
        .await
        .context("Failed to get body of responsr")?;

    Ok(public_ip)
}

pub fn resolve(hostname: &str) -> Result<String> {
    let sock_addr = format!("{}:443", hostname)
        .to_socket_addrs()
        .context(format!("Failed to resolve domain {}", hostname))?
        .next()
        .context("Resolved domain is None")?;
    Ok(sock_addr.ip().to_string())
}
