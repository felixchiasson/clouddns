use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Config {
    #[validate(length(min = 1, message = "API token cannot be empty"))]
    pub api_token: Cow<'static, str>,

    #[validate(length(min = 1, message = "Zone ID cannot be empty"))]
    pub zone_id: Cow<'static, str>,

    #[validate(range(min = 1, message = "Update interval must be greater than 0"))]
    pub update_interval: u64,

    #[validate(nested)]
    pub domain_list: Vec<Domain>,

    #[validate(range(min = 1, message = "TTL must be greater than 0"))]
    pub record_ttl: u32,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Domain {
    #[validate(length(min = 1, message = "Domain name cannot be empty"))]
    pub name: Cow<'static, str>,

    #[validate(length(min = 1, message = "Record cannot be empty"))]
    pub record: Cow<'static, str>,
}
