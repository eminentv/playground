# Azure Search 
## Setup
cargo install rust-script
## Supported Resource Types
### Network Resources
Virtual Networks: network, networks, vnet, vnets
Network Security Groups: nsg, nsgs
Public IP Addresses: publicip, publicips
Network Interfaces: nic, nics
Load Balancers: loadbalancer, loadbalancers
### Compute Resources
Virtual Machines: vm, vms
VM Scale Sets: vmss
Managed Disks: disk, disks

### Storage Resources
Storage Accounts: storage, storageaccount, storageaccounts

### Security Resources
Key Vaults: keyvault, keyvaults, kv
App Service Resources
Web Apps: webapp, webapps, appservice, appservices

### Database Resources
SQL Servers: sql, sqlserver, sqlservers
Cosmos DB: cosmosdb

### Container Resources
AKS Clusters: aks
Container Registry: acr, containerregistry

## Usage
### Get ALL resources in subscription (default behavior)
rust-script.exe .\azure_search.rs subid
### List all resource types in subscription
rust-script.exe .\azure_search.rs subid types
### List all types of a resource in a subscription
rust-script.exe .\azure_search.rs subid resourcetype
Example: rust-script.exe .\azure_search.rs subid publicips
### List ALL resources in Resource Group
rust-script.exe .\azure_search.rs subid resourcegroupname
### List all resource type in Resource Group
rust-script.exe .\azure_search.rs subid resourcegroupname types
### List all types of a resource in resource group
rust-script.exe .\azure_search.rs subid resourcegroupname resourcetype
Example: rust-script.exe .\azure_search.rs subid resourcegroupname publicips

### Search returns full json values of where item was matched, can be used anywhere
Example: 
- rust-script.exe .\azure_search.rs subid search:linux 
- rust-script.exe .\azure_search.rs subid search:sql
- rust-script.exe .\azure_search.rs subid search:8006
- rust-script.exe .\azure_search.rs subid search:192.168.0.1

### SubSearch returns matching key and value where item was matched, matches on both key or value