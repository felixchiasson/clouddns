#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use mockall::predicate::*;
    use std::fs;
    use tempfile::NamedTempFile;

    // Mock the Cloudflare API Client
    mock! {
        pub CloudflareClient {
            fn request<T>(&self, params: T) -> Result<T::Response>;
        }
    }

    // Create mock response structures
    #[derive(Debug, Clone)]
    struct MockZoneResponse {
        zones: Vec<MockZone>,
    }

    #[derive(Debug, Clone)]
    struct MockZone {
        id: String,
        name: String,
    }

    #[derive(Debug, Clone)]
    struct MockDnsRecord {
        id: String,
        name: String,
        content: String,
        proxied: Option<bool>,
    }

    #[derive(Debug, Clone)]
    struct MockDnsResponse {
        result: Vec<MockDnsRecord>,
    }

    // Helper function to create test config
    fn create_test_config() -> (NamedTempFile, Config) {
        let config_content = r#"
            api_token: "test_token"
            zone_id: "test.example.com"
            update_interval: 300
            record_ttl: 1
            domain_list:
              - name: "test.example.com"
                record: "test"
        "#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, config_content).unwrap();

        let config: Config = serde_yaml::from_str(config_content).unwrap();
        (temp_file, config)
    }

    #[tokio::test]
    async fn test_get_zone_id_with_mock() {
        let mut mock_client = MockCloudflareClient::new();
        let (_, config) = create_test_config();

        // Setup mock response
        let mock_response = MockZoneResponse {
            zones: vec![MockZone {
                id: "zone123".to_string(),
                name: "test.example.com".to_string(),
            }],
        };

        mock_client
            .expect_request::<zone::ListZones>()
            .returning(move |_| Ok(mock_response.clone()));

        let ddns = CloudflareDdns {
            config: config,
            client: mock_client,
            current_ip: None,
        };

        let result = ddns.get_zone_id().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "zone123");
    }

    #[tokio::test]
    async fn test_get_record_id_with_mock() {
        let mut mock_client = MockCloudflareClient::new();
        let (_, config) = create_test_config();

        // Setup mock response
        let mock_response = MockDnsResponse {
            result: vec![MockDnsRecord {
                id: "record123".to_string(),
                name: "test.example.com".to_string(),
                content: "1.1.1.1".to_string(),
                proxied: Some(true),
            }],
        };

        mock_client
            .expect_request::<dns::ListDnsRecords>()
            .returning(move |_| Ok(mock_response.clone()));

        let ddns = CloudflareDdns {
            config: config.clone(),
            client: mock_client,
            current_ip: None,
        };

        let domain = &config.domain_list[0];
        let result = ddns.get_record_id("zone123", domain).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "record123");
    }

    #[tokio::test]
    async fn test_get_record_proxy_status_with_mock() {
        let mut mock_client = MockCloudflareClient::new();
        let (_, config) = create_test_config();

        // Setup mock response
        let mock_response = MockDnsRecord {
            id: "record123".to_string(),
            name: "test.example.com".to_string(),
            content: "1.1.1.1".to_string(),
            proxied: Some(true),
        };

        mock_client
            .expect_request::<dns::GetDnsRecord>()
            .returning(move |_| Ok(mock_response.clone()));

        let ddns = CloudflareDdns {
            config: config,
            client: mock_client,
            current_ip: None,
        };

        let result = ddns
            .get_record_proxy_status("zone123", "record123", "test.example.com")
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[tokio::test]
    async fn test_update_record_with_mock() {
        let mut mock_client = MockCloudflareClient::new();
        let (_, config) = create_test_config();

        // Setup mock responses for the chain of calls
        mock_client
            .expect_request::<dns::ListDnsRecords>()
            .returning(|_| {
                Ok(MockDnsResponse {
                    result: vec![MockDnsRecord {
                        id: "record123".to_string(),
                        name: "test.example.com".to_string(),
                        content: "1.1.1.1".to_string(),
                        proxied: Some(true),
                    }],
                })
            });

        mock_client
            .expect_request::<dns::GetDnsRecord>()
            .returning(|_| {
                Ok(MockDnsRecord {
                    id: "record123".to_string(),
                    name: "test.example.com".to_string(),
                    content: "1.1.1.1".to_string(),
                    proxied: Some(true),
                })
            });

        mock_client
            .expect_request::<dns::UpdateDnsRecord>()
            .returning(|_| {
                Ok(MockDnsRecord {
                    id: "record123".to_string(),
                    name: "test.example.com".to_string(),
                    content: "2.2.2.2".to_string(),
                    proxied: Some(true),
                })
            });

        let ddns = CloudflareDdns {
            config: config,
            client: mock_client,
            current_ip: None,
        };

        let result = ddns
            .update_record("zone123", "2.2.2.2", "test.example.com")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_all_records_with_mock() {
        let mut mock_client = MockCloudflareClient::new();
        let (_, config) = create_test_config();

        // Mock get_current_ip
        mock_client
            .expect_request::<reqwest::Response>()
            .returning(|_| {
                Ok(TraceResponse {
                    ip: "2.2.2.2".to_string(),
                })
            });

        // Mock get_zone_id
        mock_client
            .expect_request::<zone::ListZones>()
            .returning(|_| {
                Ok(MockZoneResponse {
                    zones: vec![MockZone {
                        id: "zone123".to_string(),
                        name: "test.example.com".to_string(),
                    }],
                })
            });

        // Mock the update chain
        mock_client
            .expect_request::<dns::ListDnsRecords>()
            .returning(|_| {
                Ok(MockDnsResponse {
                    result: vec![MockDnsRecord {
                        id: "record123".to_string(),
                        name: "test.example.com".to_string(),
                        content: "1.1.1.1".to_string(),
                        proxied: Some(true),
                    }],
                })
            });

        mock_client
            .expect_request::<dns::GetDnsRecord>()
            .returning(|_| {
                Ok(MockDnsRecord {
                    id: "record123".to_string(),
                    name: "test.example.com".to_string(),
                    content: "1.1.1.1".to_string(),
                    proxied: Some(true),
                })
            });

        mock_client
            .expect_request::<dns::UpdateDnsRecord>()
            .returning(|_| {
                Ok(MockDnsRecord {
                    id: "record123".to_string(),
                    name: "test.example.com".to_string(),
                    content: "2.2.2.2".to_string(),
                    proxied: Some(true),
                })
            });

        let mut ddns = CloudflareDdns {
            config: config,
            client: mock_client,
            current_ip: None,
        };

        let result = ddns.update_all_records().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_all_records_no_ip_change() {
        let mut mock_client = MockCloudflareClient::new();
        let (_, config) = create_test_config();

        let ddns = CloudflareDdns {
            config: config,
            client: mock_client,
            current_ip: Some("2.2.2.2".to_string()),
        };

        // Mock get_current_ip to return the same IP
        mock_client
            .expect_request::<reqwest::Response>()
            .returning(|_| {
                Ok(TraceResponse {
                    ip: "2.2.2.2".to_string(),
                })
            });

        let result = ddns.update_all_records().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_deserialization() {
        let config_str = r#"
            api_token: "test_token"
            zone_id: "test.example.com"
            update_interval: 300
            record_ttl: 1
            domain_list:
              - name: "test.example.com"
                record: "test"
        "#;

        let config: Result<Config, _> = serde_yaml::from_str(config_str);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.api_token, "test_token");
        assert_eq!(config.zone_id, "test.example.com");
        assert_eq!(config.update_interval, 300);
        assert_eq!(config.record_ttl, 1);
        assert_eq!(config.domain_list.len(), 1);
        assert_eq!(config.domain_list[0].name, "test.example.com");
        assert_eq!(config.domain_list[0].record, "test");
    }

    #[test]
    fn test_invalid_config() {
        let invalid_config = r#"
            api_token: "test_token"
            # missing required fields
        "#;

        let config: Result<Config, _> = serde_yaml::from_str(invalid_config);
        assert!(config.is_err());
    }
}
