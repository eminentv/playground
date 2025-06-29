locals {
  # CoreDNS configuration with dynamic upstream servers
  coredns_config = templatefile("${path.module}/coredns.conf.tpl", {
    upstream_servers = join(" ", var.upstream_dns_servers)
  })

  # common tags applied to all resources
  common_tags = merge(
    {
      Environment   = var.environment
      Project       = "CoreDNS-Challenge"
      ManagedBy     = "Terraform"
      DeployedDate  = formatdate("YYYY-MM-DD", timestamp())
    },
    var.tags
  )

  # network configuration
  subnet_cidrs = [
    for i in range(var.zone_count) : 
    cidrsubnet(var.vnet_address_space[0], 8, i + 1)
  ]

  # container configuration
  container_ports = [
    {
      port     = 53
      protocol = "UDP"
      name     = "dns-udp"
    },
    {
      port     = 53
      protocol = "TCP"
      name     = "dns-tcp"
    },
    {
      port     = 8080
      protocol = "TCP"
      name     = "health"
    },
    {
      port     = 8181
      protocol = "TCP"
      name     = "ready"
    },
    {
      port     = 9153
      protocol = "TCP"
      name     = "metrics"
    }
  ]

  # naming conventions
  resource_names = {
    vnet               = "vnet-coredns-${var.environment}"
    nsg                = "nsg-coredns-${var.environment}"
    lb                 = "lb-coredns-${var.environment}"
    pip                = "pip-coredns-lb-${var.environment}"
    acr                = "acrcoredns${var.environment}${random_string.suffix.result}"
    container_group    = "aci-coredns-${var.environment}"
  }

  # zone mapping for availability zones
  zone_names = {
    "1" = "zone-a"
    "2" = "zone-b" 
    "3" = "zone-c"
    "4" = "zone-d"
    "5" = "zone-e"
  }
}