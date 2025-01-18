use crate::api::{CloudflareClient, DnsApiClient};
use crate::config::Config;
use anyhow::{Context, Result};
use log::{error, info};
use serde::Deserialize;
use std::{fs::File, future::Future, io::Read, net::Ipv4Addr, str::FromStr};
use tokio::signal;
use tokio::time::{sleep, Duration};

const IP_CHECK_URL: &str = "https://api64.ipify.org?format=json";

// Establish the structure of the Domain as it pertains to the Cloudflare API

#[derive(Debug, Deserialize)]
struct TraceResponse {
    ip: String,
}

pub struct CloudflareDdns {
    config: Config,
    api_client: Box<dyn DnsApiClient>,
    current_ip: Option<Ipv4Addr>,
}

impl CloudflareDdns {
    pub async fn new(config_file: &str) -> Result<Self> {
        let config = Self::load_config(config_file)?;
        let api_client = Box::new(CloudflareClient::new(config.api_token.clone()));

        Ok(Self {
            config,
            api_client,
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
        let response = reqwest::get(IP_CHECK_URL)
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

    async fn update_all_records(&mut self) -> Result<(), anyhow::Error> {
        let current_ip = self.get_current_ip().await?;
        let zone_id = self.config.zone_id.clone();

        self.current_ip = Some(current_ip);
        info!("Current IP: {}", current_ip);

        for domain in &self.config.domain_list {
            info!("Updating record for: {}", domain.record);

            let record = self.api_client.get_record(&zone_id, &domain.name).await?;

            if record.content == current_ip.to_string() {
                info!("Record already up to date");
                continue;
            }

            match self
                .api_client
                .update_record(&zone_id, &record, &current_ip, self.config.record_ttl)
                .await
            {
                Ok(_) => {
                    info!("Record updated successfully");
                }
                Err(e) => {
                    error!("Failed to update record: {}", e);
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    pub async fn shutdown_signal() {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("Failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => println!("Received Ctrl+C signal"),
            _ = terminate => println!("Received termination signal"),
        }
    }

    pub async fn run(&mut self, shutdown: impl Future<Output = ()>) -> Result<()> {
        let interval = Duration::from_secs(self.config.update_interval * 60);

        if let Err(e) = self.update_all_records().await {
            error!("Error during initial update: {}", e);
        }

        tokio::pin!(shutdown);

        loop {
            tokio::select! {
                _ = &mut shutdown => {
                    info!("Shutdown signal received");
                    break;
                }
                _ = sleep(interval) => {
                    if let Err(e) = self.update_all_records().await {
                        error!("Error updating records: {}", e);
                    }
                }
            }
        }
        Ok(())
    }
}
