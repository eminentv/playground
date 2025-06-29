# resource group information
output "resource_group_name" {
  description = "Name of the resource group"
  value       = azurerm_resource_group.main.name
}

output "resource_group_location" {
  description = "Location of the resource group"
  value       = azurerm_resource_group.main.location
}

# container information
output "container_group_names" {
  description = "Names of the CoreDNS container groups"
  value       = azurerm_container_group.coredns[*].name
}

output "container_group_fqdns" {
  description = "FQDNs of the CoreDNS container groups"
  value       = azurerm_container_group.coredns[*].fqdn
}

output "container_group_ips" {
  description = "Public IPs of the CoreDNS container groups"
  value       = azurerm_container_group.coredns[*].ip_address
}

output "container_group_zones" {
  description = "Availability zones for each container group"
  value = {
    for i, cg in azurerm_container_group.coredns : 
    cg.name => cg.zones[0]
  }
}

# load balancer information
output "load_balancer_name" {
  description = "Name of the load balancer"
  value       = azurerm_lb.coredns.name
}

output "load_balancer_ip" {
  description = "Public IP of the load balancer"
  value       = azurerm_public_ip.lb.ip_address
}

output "load_balancer_fqdn" {
  description = "FQDN of the load balancer"
  value       = azurerm_public_ip.lb.fqdn
}

# network information
output "virtual_network_name" {
  description = "Name of the virtual network"
  value       = azurerm_virtual_network.main.name
}

output "subnet_ids" {
  description = "IDs of the subnets"
  value       = azurerm_subnet.zone_subnets[*].id
}

# container registry information
output "container_registry_name" {
  description = "Name of the container registry (if created)"
  value       = var.enable_container_registry ? azurerm_container_registry.main[0].name : null
}

output "container_registry_login_server" {
  description = "Login server of the container registry (if created)"
  value       = var.enable_container_registry ? azurerm_container_registry.main[0].login_server : null
}

# DNS Testing information
output "dns_test_commands" {
  description = "Commands to test DNS resolution"
  value = [
    "nslookup google.com ${azurerm_public_ip.lb.ip_address}",
    "dig @${azurerm_public_ip.lb.ip_address} google.com",
    "host google.com ${azurerm_public_ip.lb.ip_address}"
  ]
}

# health check URLs
output "health_check_urls" {
  description = "Health check URLs for each container instance"
  value = [
    for cg in azurerm_container_group.coredns :
    "http://${cg.ip_address}:8080/health"
  ]
}

output "ready_check_urls" {
  description = "Ready check URLs for each container instance"
  value = [
    for cg in azurerm_container_group.coredns :
    "http://${cg.ip_address}:8181/ready"
  ]
}

output "metrics_urls" {
  description = "Prometheus metrics URLs for each container instance"
  value = [
    for cg in azurerm_container_group.coredns :
    "http://${cg.ip_address}:9153/metrics"
  ]
}

# deployment summary 
output "deployment_summary" {
  description = "Complete summary of deployment"
  value = {
    loadbalancer = {
      component = "Azure Load Balancer"
      ip        = azurerm_public_ip.lb.ip_address
      fqdn      = azurerm_public_ip.lb.fqdn
      ports     = ["53/UDP", "53/TCP", "9153/TCP"]
    }
    application = {
      component = "CoreDNS Container Instances"
      instances = {
        for i, cg in azurerm_container_group.coredns :
        "zone-${i + 1}" => {
          name = cg.name
          ip   = cg.ip_address
          fqdn = cg.fqdn
          zone = cg.zones[0]
        }
      }
      total_instances = length(azurerm_container_group.coredns)
    }
    data = {
      component        = "Upstream DNS Servers"
      upstream_servers = var.upstream_dns_servers
      note            = "CoreDNS forwards queries to these servers"
    }
  }
}

# checks
output "check_guide" {
  description = "commands to verify deployment"
  value = {
    test_dns_resolution = "nslookup google.com ${azurerm_public_ip.lb.ip_address}"
    check_health       = "curl http://${azurerm_container_group.coredns[0].ip_address}:8080/health"
    view_metrics       = "curl http://${azurerm_container_group.coredns[0].ip_address}:9153/metrics"
    list_containers    = "az container list --resource-group ${azurerm_resource_group.main.name} --output table"
  }
}