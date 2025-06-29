
# container registry
resource "azurerm_container_registry" "main" {
  count = var.enable_container_registry ? 1 : 0
  
  name                = "acrcoredns${random_string.suffix.result}"
  resource_group_name = azurerm_resource_group.main.name
  location            = azurerm_resource_group.main.location
  sku                 = "Basic"
  admin_enabled       = true

  tags = merge(
    {
      Environment = var.environment
      Component   = "container-registry"
    },
    var.tags
  )
}

# container groups - one per zone
resource "azurerm_container_group" "coredns" {
  count = var.zone_count
  
  name                = "aci-coredns-zone-${count.index + 1}"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  ip_address_type     = "Public"
  dns_name_label      = "coredns-zone-${count.index + 1}-${random_string.suffix.result}"
  os_type             = "Linux"
  restart_policy      = "Always"

  # Deploy to availability zone
  zones = [tostring(count.index + 1)]

  container {
    name   = "coredns"
    image  = var.coredns_image
    cpu    = var.container_cpu
    memory = var.container_memory

    # DNS UDP port
    ports {
      port     = 53
      protocol = "UDP"
    }

    # DNS TCP port
    ports {
      port     = 53
      protocol = "TCP"
    }

    # Health check port
    ports {
      port     = 8080
      protocol = "TCP"
    }

    # ready check port
    ports {
      port     = 8181
      protocol = "TCP"
    }

    # Metrics port
    ports {
      port     = 9153
      protocol = "TCP"
    }

    # coreDNS configuration
    environment_variables = {
      COREDNS_CONFIG = base64encode(local.coredns_config)
    }

    # liveness probe
    liveness_probe {
      http_get {
        path   = "/health"
        port   = 8080
        scheme = "Http"
      }
      initial_delay_seconds = 30
      period_seconds        = 10
      timeout_seconds       = 5
      failure_threshold     = 3
      success_threshold     = 1
    }

    # readiness probe
    readiness_probe {
      http_get {
        path   = "/ready"
        port   = 8181
        scheme = "Http"
      }
      initial_delay_seconds = 10
      period_seconds        = 5
      timeout_seconds       = 3
      failure_threshold     = 3
      success_threshold     = 1
    }
  }

  tags = merge(
    {
      Environment = var.environment
      Component   = "container-instance"
      Zone        = "zone-${count.index + 1}"
      app        = "application"
    },
    var.tags
  )
}