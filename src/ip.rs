use std::error::Error;
use std::net::ToSocketAddrs;
use tracing::{debug, warn};

const PIP_PROVIDERS: [&str; 5] = [
    "https://ifconfig.me/ip",
    "https://icanhazip.com",
    "https://myip.dnsomatic.com",
    "https://myexternalip.com/raw",
    "https://api.ipify.org",
];

pub async fn get_pip() -> Result<String, Box<dyn Error>> {
    let mut pip: String = String::from("");
    for url in PIP_PROVIDERS.iter() {
        if let Ok(response) = reqwest::get(url.to_string()).await {
            pip = response.text().await?;
            debug!("Fetched IP: {pip} from {url}");
            break;
        }
        warn!("Failed to fetch IP from {url}");
    }
    Ok(pip)
}

pub fn resolve(hostname: &str) -> Result<String, Box<dyn Error>> {
    let sock_addr = format!("{hostname}:443")
        .to_socket_addrs()?
        .next()
        .expect("Cannot resolve hostname");
    Ok(sock_addr.ip().to_string())
}
