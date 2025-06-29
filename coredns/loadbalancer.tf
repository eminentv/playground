# public IP for load balancer
resource "azurerm_public_ip" "lb" {
  name                = "pip-coredns-lb"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  allocation_method   = "Static"
  sku                 = "Standard"
  zones               = ["1", "2", "3"]  # Zone-redundant

  tags = merge(
    {
      Environment = var.environment
      Component   = "load-balancer"
      app        = "network"
    },
    var.tags
  )
}

# load balancer
resource "azurerm_lb" "coredns" {
  name                = "lb-coredns"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  sku                 = "Standard"

  frontend_ip_configuration {
    name                 = "coredns-frontend"
    public_ip_address_id = azurerm_public_ip.lb.id
  }

  tags = merge(
    {
      Environment = var.environment
      Component   = "load-balancer"
      Tier        = "presentation"
    },
    var.tags
  )
}

# backend address pool
resource "azurerm_lb_backend_address_pool" "coredns" {
  loadbalancer_id = azurerm_lb.coredns.id
  name            = "coredns-backend-pool"
}

# health probe
resource "azurerm_lb_probe" "coredns_health" {
  loadbalancer_id = azurerm_lb.coredns.id
  name            = "coredns-health-probe"
  port            = 8080
  protocol        = "Http"
  request_path    = "/health"
  interval_in_seconds = 15
  number_of_probes    = 2
}

# ready probe
resource "azurerm_lb_probe" "coredns_ready" {
  loadbalancer_id = azurerm_lb.coredns.id
  name            = "coredns-ready-probe"
  port            = 8181
  protocol        = "Http"
  request_path    = "/ready"
  interval_in_seconds = 15
  number_of_probes    = 2
}

# load balancing rule for DNS UDP
resource "azurerm_lb_rule" "dns_udp" {
  loadbalancer_id                = azurerm_lb.coredns.id
  name                           = "dns-udp-rule"
  protocol                       = "Udp"
  frontend_port                  = 53
  backend_port                   = 53
  frontend_ip_configuration_name = "coredns-frontend"
  backend_address_pool_ids       = [azurerm_lb_backend_address_pool.coredns.id]
  probe_id                       = azurerm_lb_probe.coredns_health.id
  load_distribution              = "SourceIP"
  enable_floating_ip             = false
  idle_timeout_in_minutes        = 4
}

# load balancing rule for DNS TCP
resource "azurerm_lb_rule" "dns_tcp" {
  loadbalancer_id                = azurerm_lb.coredns.id
  name                           = "dns-tcp-rule"
  protocol                       = "Tcp"
  frontend_port                  = 53
  backend_port                   = 53
  frontend_ip_configuration_name = "coredns-frontend"
  backend_address_pool_ids       = [azurerm_lb_backend_address_pool.coredns.id]
  probe_id                       = azurerm_lb_probe.coredns_health.id
  load_distribution              = "Default"
  enable_floating_ip             = false
  idle_timeout_in_minutes        = 4
}

# load balancing rule for metrics
resource "azurerm_lb_rule" "metrics" {
  loadbalancer_id                = azurerm_lb.coredns.id
  name                           = "metrics-rule"
  protocol                       = "Tcp"
  frontend_port                  = 9153
  backend_port                   = 9153
  frontend_ip_configuration_name = "coredns-frontend"
  backend_address_pool_ids       = [azurerm_lb_backend_address_pool.coredns.id]
  probe_id                       = azurerm_lb_probe.coredns_ready.id
  load_distribution              = "Default"
  enable_floating_ip             = false
  idle_timeout_in_minutes        = 4
}

# NAT rules for direct access to individual instances (for debugging)
resource "azurerm_lb_nat_rule" "coredns_ssh" {
  count = var.zone_count
  
  resource_group_name            = azurerm_resource_group.main.name
  loadbalancer_id                = azurerm_lb.coredns.id
  name                           = "coredns-direct-${count.index + 1}"
  protocol                       = "Tcp"
  frontend_port                  = 8080 + count.index
  backend_port                   = 8080
  frontend_ip_configuration_name = "coredns-frontend"
}