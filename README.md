# DDNS in Rust for Cloudflare

This is an attempt to force myself to be productive and learn Rust. If you come across this repository, somehow, please be critical of its contents.

config.toml
```
api_token = "token_here"
update_interval = 5                                    # minutes
record_ttl = 120                                       # seconds

[[zones]]
id = "zone_id"

[[zones.domains]]
name = "domain.name"
records = ["@", "subdomain"]

[[zones.domains]]
name = "domain2.name"
records = ["@", "subdomain"]



```
