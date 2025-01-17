# DDNS in Rust for Cloudflare
I was bored.

config.yaml
```
api_token: "api_token"
update_interval: 5 # minutes
proxied: true
record_ttl: 1
zone_id: "zone_id"
domain_list:
  - name: "domain.tld"
    record: "subdomain.tld"
  - name: "domain.org"
    record: "domain.org"

```
