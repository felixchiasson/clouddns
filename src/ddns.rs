use anyhow::{Context, Result};
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::{self, json};
use std::{fs::File, io::Read, net::Ipv4Addr, str::FromStr};
use tokio::time::{sleep, Duration};

// Establish the structure of the Domain as it pertains to the Cloudflare API

#[derive(Debug, Serialize, Deserialize)]
struct Domain {
    name: String,
    record: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    pub api_token: String,
    pub zone_id: String,
    pub update_interval: u64,
    pub domain_list: Vec<Domain>,
    pub record_ttl: u32,
}

#[derive(Debug, Deserialize)]
struct TraceResponse {
    ip: String,
}

#[derive(Debug, Deserialize)]
struct ApiDnsRecord {
    id: String,
    name: String,
    content: String,
    #[serde(default)]
    proxied: bool,
    ttl: u32,
}

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    result: T,
    success: bool,
    #[serde(default)]
    errors: Vec<serde_json::Value>,
    #[serde(default)]
    messages: Vec<serde_json::Value>,
}

pub struct CloudflareDdns {
    config: Config,
    client: reqwest::Client,
    current_ip: Option<Ipv4Addr>,
}

impl CloudflareDdns {
    pub async fn new(config_file: &str) -> Result<Self> {
        let config = Self::load_config(config_file)?;
        let client = reqwest::Client::new();

        Ok(Self {
            config,
            client,
            current_ip: None,
        })
    }

    fn load_config(config_file: &str) -> Result<Config> {
        let mut file = File::open(config_file)
            .with_context(|| format!("Failed to open config file: {}", config_file))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .with_context(|| format!("Failed to read config file: {}", config_file))?;

        serde_yaml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", config_file))
    }

    // Using ipify to get the current IP address, seems to be the one with the least restrictions
    async fn get_current_ip(&self) -> Result<Ipv4Addr, anyhow::Error> {
        let response = reqwest::get("https://api64.ipify.org?format=json")
            .await?
            .json::<TraceResponse>()
            .await?;

        let ipv4_response = Ipv4Addr::from_str(&response.ip);

        // Handle error if the response is empty
        match ipv4_response {
            Ok(ip) => Ok(ip),
            Err(e) => Err(anyhow::anyhow!("Failed to parse IP address: {}", e)),
        }
    }

    async fn get_record_content(&self, zone_id: &str, domain: &Domain) -> Result<ApiDnsRecord> {
        let response = self
            .client
            .get(&format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
                zone_id
            ))
            .bearer_auth(self.config.api_token.clone())
            .header("Content-Type", "application/json")
            .send()
            .await?;

        let text = response.text().await?;

        let parsed: ApiResponse<Vec<ApiDnsRecord>> = serde_json::from_str(&text).map_err(|e| {
            anyhow::anyhow!("Failed to parse API response: {}. Response: {}", e, text)
        })?;

        if !parsed.success {
            return Err(anyhow::anyhow!("API request failed: {:?}", parsed.errors));
        }

        parsed
            .result
            .into_iter()
            .find(|record| record.name == domain.name)
            .ok_or_else(|| anyhow::anyhow!("DNS record not found for domain: {}", domain.name))
    }

    async fn update_record(
        &self,
        zone_id: &str,
        ip: &Ipv4Addr,
        domain: &Domain,
    ) -> Result<(), anyhow::Error> {
        let record_content = self.get_record_content(zone_id, domain).await?;

        if record_content.content == ip.to_string() {
            info!("Record already up to date");
            return Ok(());
        }

        let response = self
            .client
            .patch(&format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                zone_id, record_content.id
            ))
            .bearer_auth(self.config.api_token.clone())
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&json!({
                "type": "A", // this can be programmed also?
                "name": domain.record,
                "content": ip.to_string(),
                "ttl": self.config.record_ttl,
                "proxied": record_content.proxied, // keep the current conf
            }))?)
            .send()
            .await?;

        let text = response.text().await?;
        info!("Update Response: {}", text);

        let update_response: ApiResponse<ApiDnsRecord> =
            serde_json::from_str(&text).map_err(|e| {
                anyhow::anyhow!("Failed to parse update response: {}. Response: {}", e, text)
            })?;

        if !update_response.success {
            return Err(anyhow::anyhow!(
                "Failed to update DNS record: {:?}",
                update_response.errors
            ));
        }

        info!("Record updated successfully");
        Ok(())
    }

    async fn update_all_records(&mut self) -> Result<(), anyhow::Error> {
        let current_ip = self.get_current_ip().await?;
        let zone_id = self.config.zone_id.clone();

        self.current_ip = Some(current_ip.clone());
        info!("Current IP: {}", current_ip);

        for domain in &self.config.domain_list {
            info!("Updating record for: {}", domain.record);
            match self.update_record(&zone_id, &current_ip, domain).await {
                Ok(_) => {
                    info!("Done.");
                }
                Err(e) => {
                    error!("Failed to update record: {}", e);
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), anyhow::Error> {
        let interval = Duration::from_secs(self.config.update_interval * 60);

        loop {
            if let Err(e) = self.update_all_records().await {
                error!("Error updating records: {}", e);
            }
            sleep(interval).await;
        }
    }
}
