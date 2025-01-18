use super::models::*;
use anyhow::Result;
use async_trait::async_trait;
use std::net::Ipv4Addr;

#[async_trait]
pub trait DnsApiClient {
    async fn get_record(&self, zone_id: &str, domain: &str) -> Result<DnsRecordUpdate>;
    async fn update_record(
        &self,
        zone_id: &str,
        record: &DnsRecordUpdate,
        content: &Ipv4Addr,
        ttl: u32,
    ) -> Result<ApiDnsRecord>;
}
