# DDNS in Rust for Cloudflare

This is an attempt to force myself to be productive and learn Rust. If you come across this repository, somehow, please be critical of its contents.

config.yaml
```
api_token: "api_token"
update_interval: 5 # minutes
proxied: true # not currently implemented, uses the current proxy settings in Cloudflare
record_ttl: 1
zone_id: "zone_id"
domain_list:
  - name: "domain.tld"
    record: "subdomain.domain.tld"
  - name: "domain.org"
    record: "domain.org"

```
