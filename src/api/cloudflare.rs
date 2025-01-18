use std::net::Ipv4Addr;

use super::{client::DnsApiClient, models::*};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;

const API_BASE_URL: &str = "https://api.cloudflare.com/client/v4";

pub struct CloudflareClient {
    client: reqwest::Client,
    api_token: String,
}

#[async_trait]
impl DnsApiClient for CloudflareClient {
    async fn get_record(&self, zone_id: &str, domain: &str) -> Result<DnsRecordUpdate> {
        let response = self
            .client
            .get(&format!("{}/zones/{}/dns_records", API_BASE_URL, zone_id))
            .headers(self.build_headers())
            .send()
            .await?;

        let response_json: ApiResponse<Vec<DnsRecordUpdate>> = response.json().await?;
        let record = response_json
            .result
            .into_iter()
            .find(|record| record.name == domain)
            .ok_or_else(|| anyhow::anyhow!("DNS record not found for domain: {}", domain))?;

        Ok(record)
    }

    async fn update_record(
        &self,
        zone_id: &str,
        record: &DnsRecordUpdate,
        content: &Ipv4Addr,
        ttl: u32,
    ) -> Result<ApiDnsRecord> {
        let response = self
            .client
            .patch(&format!(
                "{}/zones/{}/dns_records/{}",
                API_BASE_URL, zone_id, record.id
            ))
            .bearer_auth(&self.api_token)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&json!({
                "type": record.r#type,
                "name": record.name,
                "content": content.to_string(),
                "ttl": ttl,
                "proxied": record.proxied,
            }))?)
            .send()
            .await?;

        // Handle response
        let text = response.text().await?;
        let update_response: ApiResponse<ApiDnsRecord> = serde_json::from_str(&text)?;

        if !update_response.success {
            return Err(anyhow::anyhow!(
                "Failed to update DNS record: {:?}",
                update_response.errors
            ));
        }

        Ok(update_response.result)
    }
}

impl CloudflareClient {
    pub fn new(api_token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_token,
        }
    }

    fn build_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", self.api_token).parse().unwrap(),
        );
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers
    }
}
