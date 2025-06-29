# Basic Configuration
resource_group_name = "rg-coredns-challenge"
location           = "East US"
environment        = "dev"

# Deployment Configuration
zone_count = 3

# Container Configuration
coredns_image      = "coredns/coredns:1.10.1"
container_cpu      = "0.5"
container_memory   = "1.0"

# Network Configuration
vnet_address_space = ["10.0.0.0/16"]

# DNS Configuration
upstream_dns_servers = ["8.8.8.8", "8.8.4.4"]

# Optional Features
enable_container_registry = true

# Additional Tags
tags = {
  Owner      = "DevOps Team"
  CostCenter = "Engineering"
  Project    = "DNS Prod"
}