use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiDnsRecord {
    pub id: String,
    pub name: String,
    pub content: String,
    pub r#type: String,
    #[serde(default)]
    pub proxied: bool,
    pub ttl: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DnsRecordUpdate {
    pub id: String,
    pub name: String,
    pub content: String,
    pub ttl: u32,
    pub proxied: bool,
    pub r#type: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub result: T,
    pub success: bool,
    #[serde(default)]
    pub errors: Vec<serde_json::Value>,
}
