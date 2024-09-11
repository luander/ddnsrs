use crate::{cli::Cli, MessageReceiver, ShutdownReceiver};
use anyhow::{Context, Result};
use reqwest::header;
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use tokio::sync::broadcast::error::RecvError;
use tracing::{error, info};

const CF_API: &str = "https://api.cloudflare.com/client/v4";

pub async fn run(
    config: Cli,
    mut messages: MessageReceiver,
    mut shutdown: ShutdownReceiver,
) -> Result<()> {
    loop {
        tokio::select! {
            _ = shutdown.recv() => return Ok(()),
            message = messages.recv() => match message {
                Ok(message) => {
                    let cf_client = CFClient::new(&config.cloudflare_key);
                    let hostname = &config.hostname;
                    let dns_record = cf_client.dns_record(hostname).await?;
                    let public_ip = message.ip;
                    let current_ip = &dns_record["content"];
                    info!("{hostname}: current IP: {current_ip}, public IP: {public_ip}");
                    if !public_ip.eq(current_ip) {
                        cf_client.update_record(hostname, &dns_record, &public_ip).await
                            .context(format!("unable to update {hostname} A record to {public_ip}"))?;
                    }
                },
                Err(RecvError::Closed) => {
                    error!("Error receiving message: channel is closed");
                    return Err(anyhow::anyhow!("Error receiving message: channel is closed"));
                },
                Err(RecvError::Lagged(lagged)) => {
                    error!("Error receiving message: lagged {} messages", lagged);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CFClient {
    client: Client,
}

impl CFClient {
    pub fn new(token: &str) -> Self {
        let mut headers = header::HeaderMap::new();
        let mut auth_token =
            header::HeaderValue::from_bytes(format!("Bearer {}", token).as_bytes())
                .context("unable to set authentication header")
                .unwrap();
        auth_token.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, auth_token);
        let client = Client::builder()
            .default_headers(headers)
            .build()
            .context("failed to create reqwest client")
            .unwrap();
        CFClient { client }
    }

    pub async fn zone_id(&self, zone: &String) -> Result<String> {
        let resp = self
            .client
            .get(format!("{}/zones?name={}&status=active", CF_API, zone))
            .send()
            .await?;
        let result = resp.json::<Value>().await?;
        let id = &result["result"][0]["id"];
        info!("zone_id: {:#?}", id);
        let Some(id) = id.as_str() else {
            return Err(anyhow::anyhow!("unable to get zone id for {}", zone));
        };
        Ok(id.to_owned())
    }

    pub async fn dns_record(&self, hostname: &String) -> Result<Value> {
        let zone_name = self.get_fqdn(hostname);
        let zone_id = self.zone_id(&zone_name).await?;
        let url = format!(
            "{}/zones/{}/dns_records?name={}&type=A",
            CF_API, zone_id, hostname
        );
        info!("url: {}", url);
        let resp = self.client.get(url).send().await?;
        let result = resp.json::<Value>().await?;
        let id = &result["result"][0];
        Ok(id.to_owned())
    }

    pub fn get_fqdn(&self, hostname: &String) -> String {
        let split: Vec<&str> = hostname.split('.').collect();
        let fqdn = split.as_slice()[split.len() - 2..].join(".");
        fqdn
    }

    pub async fn update_record(
        self,
        hostname: &String,
        dns_record: &Value,
        ip: &String,
    ) -> Result<()> {
        let Some(zone_id) = dns_record["zone_id"].as_str() else {
            return Err(anyhow::anyhow!("unable to get zone id for {}", hostname));
        };
        let Some(record_id) = dns_record["id"].as_str() else {
            return Err(anyhow::anyhow!("unable to get record id for {}", hostname));
        };
        let url = format!("{}/zones/{}/dns_records/{}", CF_API, zone_id, record_id);
        info!("url: {}", url);
        let request_body = json!({
            "id": record_id,
            "type": "A",
            "name": hostname,
            "content": ip,
            "ttl": 60*5 // 5 minutes TTL
        });
        let resp = self.client.put(url).json(&request_body).send().await?;

        if resp.status() == StatusCode::OK {
            let body = resp.json::<Value>().await?;
            info!("{} updated to {}: {:#?}", hostname, ip, body);
        } else {
            let body = resp.json::<Value>().await?;
            error!("failed to update {}: {:#?}", hostname, body)
        }
        Ok(())
    }
}
