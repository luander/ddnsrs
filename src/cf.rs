use reqwest::header;
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use std::error::Error;
use tracing::{error, info};

const CF_API: &str = "https://api.cloudflare.com/client/v4";

#[derive(Debug)]
pub struct CFClient {
    client: Client,
}

impl CFClient {
    pub fn new(token: String) -> Self {
        let mut headers = header::HeaderMap::new();
        let mut auth_token =
            header::HeaderValue::from_bytes(format!("Bearer {}", token).as_bytes()).unwrap();
        auth_token.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, auth_token);
        let client = Client::builder().default_headers(headers).build().unwrap();
        CFClient { client }
    }

    pub async fn zone_id(self: &Self, zone: &String) -> Result<String, Box<dyn Error>> {
        let resp = self
            .client
            .get(format!("{}/zones?name={}&status=active", CF_API, zone))
            .send()
            .await?;
        let result = resp.json::<Value>().await?;
        let id = &result["result"][0]["id"];
        info!("zone_id: {:#?}", id);
        Ok(id.as_str().unwrap().to_owned())
    }

    pub async fn dns_record(self: &Self, hostname: &String) -> Result<Value, Box<dyn Error>> {
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

    pub fn get_fqdn(self: &Self, hostname: &String) -> String {
        let split: Vec<&str> = hostname.split('.').collect();
        let fqdn = split.as_slice()[split.len() - 2..].join(".");
        fqdn
    }

    pub async fn update_record(
        self: Self,
        hostname: &String,
        dns_record: &Value,
        ip: &String,
    ) -> Result<(), Box<dyn Error>> {
        let zone_id = dns_record["zone_id"].as_str().unwrap();
        let record_id = dns_record["id"].as_str().unwrap();
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
