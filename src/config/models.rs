use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_token: String,
    pub zone_id: String,
    pub update_interval: u64,
    pub domain_list: Vec<Domain>,
    pub record_ttl: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Domain {
    pub name: String,
    pub record: String,
}
