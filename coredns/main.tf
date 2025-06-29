terraform {
  required_version = ">= 1.0"
  
  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~>3.0"
    }
    random = {
      source  = "hashicorp/random"
      version = "~>3.0"
    }
  }
}

provider "azurerm" {
  features {
    resource_group {
      prevent_deletion_if_contains_resources = false
    }
  }
}

# generate random suffix for unique naming
resource "random_string" "suffix" {
  length  = 6
  special = false
  upper   = false
  
  keepers = {
    # force regeneration when these values change
    resource_group = var.resource_group_name
    environment    = var.environment
  }
}

# main resource group 
resource "azurerm_resource_group" "main" {
  name     = var.resource_group_name
  location = var.location

  tags = merge(
    local.common_tags,
    {
      Purpose = "CoreDNS-3-Zone"
    }
  )

  lifecycle {
    prevent_destroy = false
  }
}

# data sources for existing resources
data "azurerm_client_config" "current" {}

data "azurerm_subscription" "current" {}