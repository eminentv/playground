# virtual network
resource "azurerm_virtual_network" "main" {
  name                = "vnet-coredns"
  address_space       = var.vnet_address_space
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name

  tags = merge(
    {
      Environment = var.environment
      Component   = "networking"
    },
    var.tags
  )
}

# subnets for each availability zone
resource "azurerm_subnet" "zone_subnets" {
  count = var.zone_count
  
  name                 = "subnet-zone-${count.index + 1}"
  resource_group_name  = azurerm_resource_group.main.name
  virtual_network_name = azurerm_virtual_network.main.name
  address_prefixes     = ["10.0.${count.index + 1}.0/24"]
}

# network security group
resource "azurerm_network_security_group" "coredns" {
  name                = "nsg-coredns"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name

  # DNS UDP traffic
  security_rule {
    name                       = "DNS-UDP"
    priority                   = 1001
    direction                  = "Inbound"
    access                     = "Allow"
    protocol                   = "Udp"
    source_port_range          = "*"
    destination_port_range     = "53"
    source_address_prefix      = "*"
    destination_address_prefix = "*"
  }

  # DNS TCP traffic
  security_rule {
    name                       = "DNS-TCP"
    priority                   = 1002
    direction                  = "Inbound"
    access                     = "Allow"
    protocol                   = "Tcp"
    source_port_range          = "*"
    destination_port_range     = "53"
    source_address_prefix      = "*"
    destination_address_prefix = "*"
  }

  # CoreDNS health endpoint
  security_rule {
    name                       = "CoreDNS-Health"
    priority                   = 1003
    direction                  = "Inbound"
    access                     = "Allow"
    protocol                   = "Tcp"
    source_port_range          = "*"
    destination_port_range     = "8080"
    source_address_prefix      = "*"
    destination_address_prefix = "*"
  }

  # CoreDNS ready endpoint
  security_rule {
    name                       = "CoreDNS-Ready"
    priority                   = 1004
    direction                  = "Inbound"
    access                     = "Allow"
    protocol                   = "Tcp"
    source_port_range          = "*"
    destination_port_range     = "8181"
    source_address_prefix      = "*"
    destination_address_prefix = "*"
  }

  # CoreDNS metrics endpoint
  security_rule {
    name                       = "CoreDNS-Metrics"
    priority                   = 1005
    direction                  = "Inbound"
    access                     = "Allow"
    protocol                   = "Tcp"
    source_port_range          = "*"
    destination_port_range     = "9153"
    source_address_prefix      = "*"
    destination_address_prefix = "*"
  }

  tags = merge(
    {
      Environment = var.environment
      Component   = "security"
    },
    var.tags
  )
}

# associate NSG with subnets
resource "azurerm_subnet_network_security_group_association" "zone_nsg" {
  count = var.zone_count
  
  subnet_id                 = azurerm_subnet.zone_subnets[count.index].id
  network_security_group_id = azurerm_network_security_group.coredns.id
}