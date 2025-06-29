variable "resource_group_name" {
  description = "Name of the resource group"
  type        = string
  default     = "rg-coredns-challenge"
}

variable "location" {
  description = "Azure region"
  type        = string
  default     = "East US"
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "dev"
}

variable "zone_count" {
  description = "Number of availability zones to deploy across"
  type        = number
  default     = 3
  
  validation {
    condition     = var.zone_count >= 1 && var.zone_count <= 5
    error_message = "Zone count must be between 1 and 5."
  }
}

variable "coredns_image" {
  description = "CoreDNS container image"
  type        = string
  default     = "coredns/coredns:1.10.1"
}

variable "container_cpu" {
  description = "CPU allocation for each container"
  type        = string
  default     = "0.5"
}

variable "container_memory" {
  description = "Memory allocation for each container in GB"
  type        = string
  default     = "1.0"
}

variable "vnet_address_space" {
  description = "Address space for the virtual network"
  type        = list(string)
  default     = ["10.0.0.0/16"]
}

variable "upstream_dns_servers" {
  description = "Upstream DNS servers for CoreDNS forwarding"
  type        = list(string)
  default     = ["8.8.8.8", "8.8.4.4"]
}

variable "enable_container_registry" {
  description = "Whether to create a container registry"
  type        = bool
  default     = true
}

variable "tags" {
  description = "Additional tags to apply to resources"
  type        = map(string)
  default     = {}
}