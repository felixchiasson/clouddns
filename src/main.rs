mod api;
mod config;
mod ddns;
use anyhow::Result;
use ddns::CloudflareDdns;
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    // Create and run the DDNS updater
    let mut ddns = CloudflareDdns::new("config.toml").await?;
    ddns.run(CloudflareDdns::shutdown_signal()).await
}
