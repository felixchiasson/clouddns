use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Config {
    #[validate(length(min = 1, message = "API token cannot be empty"))]
    pub api_token: Cow<'static, str>,

    #[validate(range(min = 1, message = "Update interval must be greater than 0"))]
    pub update_interval: u64,

    #[validate(range(min = 1, message = "TTL must be greater than 0"))]
    pub record_ttl: u32,

    #[validate(length(min = 1, message = "At least one zone is required"))]
    pub zones: Vec<Zone>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Zone {
    #[validate(length(min = 1, message = "Zone ID cannot be empty"))]
    pub id: Cow<'static, str>,

    #[validate(nested)]
    pub domains: Vec<Domain>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Domain {
    #[validate(length(min = 1, message = "Domain name cannot be empty"))]
    pub name: Cow<'static, str>,

    #[validate(length(min = 1, message = "At least one record is required"))]
    pub records: Vec<Cow<'static, str>>,
}
