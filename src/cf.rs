use reqwest::header;
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use std::error::Error;
use tracing::{error, info};

const CF_API: &str = "https://api.cloudflare.com/client/v4";

#[derive(Debug)]
pub struct CFClient {
    client: Client,
    zone_id: Option<String>,
}

impl CFClient {
    pub fn new(token: String) -> Self {
        let mut headers = header::HeaderMap::new();
        let mut auth_token =
            header::HeaderValue::from_bytes(format!("Bearer {token}").as_bytes()).unwrap();
        auth_token.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, auth_token);
        let client = Client::builder().default_headers(headers).build().unwrap();
        CFClient {
            client,
            zone_id: None,
        }
    }

    pub async fn zone_id(&mut self, hostname: &str) -> Result<String, Box<dyn Error>> {
        if let Some(id) = &self.zone_id {
            return Ok(id.clone());
        }

        let zone = self.get_fqdn(hostname);
        let resp = self
            .client
            .get(format!("{CF_API}/zones?name={zone}&status=active"))
            .send()
            .await?;
        let result = resp.json::<Value>().await?;
        let id = &result["result"][0]["id"];
        info!("zone_id: {:#?}", id);
        let id = id.as_str().unwrap().to_owned();
        self.zone_id = Some(id.clone());
        Ok(id)
    }

    pub async fn dns_record(&mut self, hostname: &String) -> Result<Value, Box<dyn Error>> {
        let zone_id = self.zone_id(hostname).await?;
        let url = format!("{CF_API}/zones/{zone_id}/dns_records?name={hostname}&type=A");
        info!("url: {}", url);
        let resp = self.client.get(url).send().await?;
        let result = resp.json::<Value>().await?;
        let id = &result["result"][0];
        Ok(id.to_owned())
    }

    pub fn get_fqdn(&self, hostname: &str) -> String {
        let split: Vec<&str> = hostname.split('.').collect();
        let fqdn = split.as_slice()[split.len() - 2..].join(".");
        fqdn
    }

    pub async fn update_record(
        mut self,
        hostname: &str,
        dns_record: &Value,
        ip: &str,
    ) -> Result<(), Box<dyn Error>> {
        let zone_id = self.zone_id(hostname).await?;
        let record_id = dns_record["id"].as_str().unwrap();
        let url = format!("{CF_API}/zones/{zone_id}/dns_records/{record_id}");
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
