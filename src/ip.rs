use std::error::Error;
use std::net::ToSocketAddrs;

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
        let response = reqwest::get(url.to_string()).await?.text().await?;
        pip = response;
        break;
    }
    Ok(pip)
}

pub fn resolve(hostname: &str) -> Result<String, Box<dyn Error>> {
    let sock_addr = format!("{}:443", hostname)
        .to_socket_addrs()?
        .next()
        .expect("Cannot resolve hostname");
    Ok(sock_addr.ip().to_string())
}
