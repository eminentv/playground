#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! tokio = { version = "1.0", features = ["full"] }
//! reqwest = { version = "0.11", features = ["json"] }
//! serde_json = "1.0"
//! ```

use std::env;
use std::collections::HashMap;
use serde_json::Value;

// Resource type mappings
fn get_resource_mappings() -> HashMap<&'static str, (&'static str, &'static str)> {
    let mut mappings = HashMap::new();
    
    // Network resources
    mappings.insert("network", ("Microsoft.Network/virtualNetworks", "2023-05-01"));
    mappings.insert("networks", ("Microsoft.Network/virtualNetworks", "2023-05-01"));
    mappings.insert("vnet", ("Microsoft.Network/virtualNetworks", "2023-05-01"));
    mappings.insert("vnets", ("Microsoft.Network/virtualNetworks", "2023-05-01"));
    mappings.insert("nsg", ("Microsoft.Network/networkSecurityGroups", "2023-05-01"));
    mappings.insert("nsgs", ("Microsoft.Network/networkSecurityGroups", "2023-05-01"));
    mappings.insert("publicip", ("Microsoft.Network/publicIPAddresses", "2023-05-01"));
    mappings.insert("publicips", ("Microsoft.Network/publicIPAddresses", "2023-05-01"));
    mappings.insert("nic", ("Microsoft.Network/networkInterfaces", "2023-05-01"));
    mappings.insert("nics", ("Microsoft.Network/networkInterfaces", "2023-05-01"));
    mappings.insert("loadbalancer", ("Microsoft.Network/loadBalancers", "2023-05-01"));
    mappings.insert("loadbalancers", ("Microsoft.Network/loadBalancers", "2023-05-01"));
    
    // Compute resources
    mappings.insert("vm", ("Microsoft.Compute/virtualMachines", "2023-03-01"));
    mappings.insert("vms", ("Microsoft.Compute/virtualMachines", "2023-03-01"));
    mappings.insert("vmss", ("Microsoft.Compute/virtualMachineScaleSets", "2023-03-01"));
    mappings.insert("disk", ("Microsoft.Compute/disks", "2023-01-02"));
    mappings.insert("disks", ("Microsoft.Compute/disks", "2023-01-02"));
    
    // Storage resources
    mappings.insert("storage", ("Microsoft.Storage/storageAccounts", "2023-01-01"));
    mappings.insert("storageaccount", ("Microsoft.Storage/storageAccounts", "2023-01-01"));
    mappings.insert("storageaccounts", ("Microsoft.Storage/storageAccounts", "2023-01-01"));
    
    // Key Vault
    mappings.insert("keyvault", ("Microsoft.KeyVault/vaults", "2023-02-01"));
    mappings.insert("keyvaults", ("Microsoft.KeyVault/vaults", "2023-02-01"));
    mappings.insert("kv", ("Microsoft.KeyVault/vaults", "2023-02-01"));
    
    // App Service
    mappings.insert("webapp", ("Microsoft.Web/sites", "2022-09-01"));
    mappings.insert("webapps", ("Microsoft.Web/sites", "2022-09-01"));
    mappings.insert("appservice", ("Microsoft.Web/sites", "2022-09-01"));
    mappings.insert("appservices", ("Microsoft.Web/sites", "2022-09-01"));
    
    // Database
    mappings.insert("sql", ("Microsoft.Sql/servers", "2022-05-01-preview"));
    mappings.insert("sqlserver", ("Microsoft.Sql/servers", "2022-05-01-preview"));
    mappings.insert("sqlservers", ("Microsoft.Sql/servers", "2022-05-01-preview"));
    mappings.insert("cosmosdb", ("Microsoft.DocumentDB/databaseAccounts", "2023-04-15"));
    
    // Container
    mappings.insert("aks", ("Microsoft.ContainerService/managedClusters", "2023-05-01"));
    mappings.insert("acr", ("Microsoft.ContainerRegistry/registries", "2023-01-01-preview"));
    mappings.insert("containerregistry", ("Microsoft.ContainerRegistry/registries", "2023-01-01-preview"));
    
    mappings
}

async fn get_azure_token() -> Result<String, String> {
    // on windows, try powershell approach first
    if cfg!(windows) {
        let result = std::process::Command::new("powershell")
            .args(&["-Command", "az account get-access-token --output json"])
            .output();
            
        if let Ok(output) = result {
            if output.status.success() {
                let token_data: Value = serde_json::from_slice(&output.stdout)
                    .map_err(|_| "Failed to parse token".to_string())?;
                return Ok(token_data["accessToken"].as_str().unwrap().to_string());
            }
        }
    }
    
    // fallback to direct commands
    let commands = [
        "az",
        "az.exe", 
        "C:\\Program Files (x86)\\Microsoft SDKs\\Azure\\CLI2\\wbin\\az.cmd",
        "C:\\Program Files\\Microsoft SDKs\\Azure\\CLI2\\wbin\\az.cmd"
    ];
    
    for cmd in &commands {
        let result = std::process::Command::new(cmd)
            .args(&["account", "get-access-token", "--output", "json"])
            .output();
            
        match result {
            Ok(output) => {
                if output.status.success() {
                    let token_data: Value = serde_json::from_slice(&output.stdout)
                        .map_err(|_| "Failed to parse token".to_string())?;
                    return Ok(token_data["accessToken"].as_str().unwrap().to_string());
                } else {
                    let error_msg = String::from_utf8_lossy(&output.stderr);
                    eprintln!("Command '{}' failed: {}", cmd, error_msg);
                    continue;
                }
            }
            Err(e) => {
                eprintln!("Failed to execute '{}': {}", cmd, e);
                continue;
            }
        }
    }
    
    Err("Azure CLI not found. Try running 'where az' to find the correct path.".to_string())
}

async fn get_resource_json(subscription: &str, rg: &str, resource_type: &str, resource_name: &str) -> Result<Value, String> {
    let token = get_azure_token().await?;
    let mappings = get_resource_mappings();
    
    let (provider_type, api_version) = mappings.get(resource_type.to_lowercase().as_str())
        .ok_or_else(|| format!("Unknown resource type: {}. Use 'types' to see available types.", resource_type))?;
    
    let url = format!(
        "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/{}/{}?api-version={}",
        subscription, rg, provider_type, resource_name, api_version
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|_| "Request failed".to_string())?;

    if !response.status().is_success() {
        return Err(format!("Failed: {}", response.status()));
    }

    response.json().await.map_err(|_| "JSON parse failed".to_string())
}

async fn list_all_resources(subscription: &str) -> Result<Value, String> {
    let token = get_azure_token().await?;
    
    let url = format!(
        "https://management.azure.com/subscriptions/{}/resources?api-version=2021-04-01",
        subscription
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|_| "Request failed".to_string())?;

    if !response.status().is_success() {
        return Err(format!("Failed: {}", response.status()));
    }

    response.json().await.map_err(|_| "JSON parse failed".to_string())
}

async fn list_resources_by_type(subscription: &str, resource_type: &str) -> Result<Value, String> {
    let token = get_azure_token().await?;
    let mappings = get_resource_mappings();
    
    let (provider_type, api_version) = mappings.get(resource_type.to_lowercase().as_str())
        .ok_or_else(|| format!("Unknown resource type: {}. Use 'types' to see available types.", resource_type))?;
    
    let url = format!(
        "https://management.azure.com/subscriptions/{}/providers/{}?api-version={}",
        subscription, provider_type, api_version
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|_| "Request failed".to_string())?;

    if !response.status().is_success() {
        return Err(format!("Failed: {}", response.status()));
    }

    response.json().await.map_err(|_| "JSON parse failed".to_string())
}

async fn list_resources_in_rg(subscription: &str, rg: &str, resource_type: &str) -> Result<Value, String> {
    let token = get_azure_token().await?;
    let mappings = get_resource_mappings();
    
    let (provider_type, api_version) = mappings.get(resource_type.to_lowercase().as_str())
        .ok_or_else(|| format!("Unknown resource type: {}. Use 'types' to see available types.", resource_type))?;
    
    let url = format!(
        "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/{}?api-version={}",
        subscription, rg, provider_type, api_version
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|_| "Request failed".to_string())?;

    if !response.status().is_success() {
        return Err(format!("Failed: {}", response.status()));
    }

    response.json().await.map_err(|_| "JSON parse failed".to_string())
}

fn get_field(data: &Value, field: &str) -> Option<Value> {
    data.get(field).cloned()
}

fn search_json(data: &Value, term: &str) -> Value {
    let term_lower = term.to_lowercase();
    let mut results = Vec::new();
    
    // if data is a list of resources (from list_resources), search each resource
    if let Some(resources) = data.get("value").and_then(|v| v.as_array()) {
        for resource in resources {
            if resource_contains_term(resource, &term_lower) {
                results.push(resource.clone());
            }
        }
        return Value::Array(results);
    }
    
    // if data is a single resource, search within it and return the whole resource if match found
    if resource_contains_term(data, &term_lower) {
        return data.clone();
    }
    
    // no matches found
    serde_json::json!([])
}

fn subsearch_json(data: &Value, term: &str) -> Value {
    let term_lower = term.to_lowercase();
    let mut results = serde_json::Map::new();
    
    // if data is a list of resources (from list_resources), search each resource
    if let Some(resources) = data.get("value").and_then(|v| v.as_array()) {
        for (resource_index, resource) in resources.iter().enumerate() {
            let resource_name = resource.get("name")
                .and_then(|n| n.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("resource_{}", resource_index));
            
            subsearch_recursive_flat(resource, &resource_name, "", &term_lower, &mut results);
        }
        return Value::Object(results);
    }
    
    // if data is a single resource, search within it
    subsearch_recursive_flat(data, "resource", "", &term_lower, &mut results);
    Value::Object(results)
}

fn subsearch_recursive_flat(data: &Value, resource_name: &str, path: &str, term: &str, results: &mut serde_json::Map<String, Value>) {
    match data {
        Value::Object(map) => {
            for (key, value) in map {
                let current_path = if path.is_empty() { 
                    format!("{}.{}", resource_name, key)
                } else { 
                    format!("{}.{}", path, key) 
                };
                
                // check if key contains search term
                if key.to_lowercase().contains(term) {
                    results.insert(current_path.clone(), value.clone());
                }
                
                // check if value (as string) contains search term
                if let Value::String(s) = value {
                    if s.to_lowercase().contains(term) {
                        results.insert(current_path.clone(), value.clone());
                    }
                }
                
                // recurse into nested objects/arrays
                subsearch_recursive_flat(value, resource_name, &current_path, term, results);
            }
        }
        Value::Array(arr) => {
            for (index, item) in arr.iter().enumerate() {
                let current_path = if path.is_empty() { 
                    format!("{}[{}]", resource_name, index)
                } else { 
                    format!("{}[{}]", path, index) 
                };
                subsearch_recursive_flat(item, resource_name, &current_path, term, results);
            }
        }
        _ => {}
    }
}

fn resource_contains_term(resource: &Value, term: &str) -> bool {
    search_recursive_bool(resource, term)
}

fn search_recursive_bool(data: &Value, term: &str) -> bool {
    match data {
        Value::Object(map) => {
            for (key, value) in map {
                // check if key contains search term
                if key.to_lowercase().contains(term) {
                    return true;
                }
                
                // check if value (as string) contains search term
                if let Value::String(s) = value {
                    if s.to_lowercase().contains(term) {
                        return true;
                    }
                }
                
                // recurse into nested objects/arrays
                if search_recursive_bool(value, term) {
                    return true;
                }
            }
        }
        Value::Array(arr) => {
            for item in arr {
                if search_recursive_bool(item, term) {
                    return true;
                }
            }
        }
        _ => {}
    }
    false
}

fn print_available_types() {
    println!("Available resource types:");
    println!();
    println!("Network:");
    println!("  network, networks, vnet, vnets - Virtual Networks");
    println!("  nsg, nsgs - Network Security Groups");
    println!("  publicip, publicips - Public IP Addresses");
    println!("  nic, nics - Network Interfaces");
    println!("  loadbalancer, loadbalancers - Load Balancers");
    println!();
    println!("Compute:");
    println!("  vm, vms - Virtual Machines");
    println!("  vmss - Virtual Machine Scale Sets");
    println!("  disk, disks - Managed Disks");
    println!();
    println!("Storage:");
    println!("  storage, storageaccount, storageaccounts - Storage Accounts");
    println!();
    println!("Security:");
    println!("  keyvault, keyvaults, kv - Key Vaults");
    println!();
    println!("App Service:");
    println!("  webapp, webapps, appservice, appservices - Web Apps");
    println!();
    println!("Database:");
    println!("  sql, sqlserver, sqlservers - SQL Servers");
    println!("  cosmosdb - Cosmos DB Accounts");
    println!();
    println!("Container:");
    println!("  aks - Azure Kubernetes Service");
    println!("  acr, containerregistry - Container Registry");
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <subscription> [all|types|resource-type|resource-group] [resource-name] [field|search:term|subsearch:term]", args[0]);
        eprintln!("Examples:");
        eprintln!("  {} 12345", args[0]);
        eprintln!("  {} 12345 all", args[0]);
        eprintln!("  {} 12345 types", args[0]);
        eprintln!("  {} 12345 network", args[0]);
        eprintln!("  {} 12345 storage", args[0]);
        eprintln!("  {} 12345 search:Standard", args[0]);
        eprintln!("  {} 12345 subsearch:size", args[0]);
        eprintln!("  {} 12345 myRG network", args[0]);
        eprintln!("  {} 12345 myRG network myVNet", args[0]);
        eprintln!("  {} 12345 myRG network myVNet name", args[0]);
        eprintln!("  {} 12345 myRG network myVNet search:subnet", args[0]);
        eprintln!("  {} 12345 myRG network myVNet subsearch:address", args[0]);
        return;
    }

    let subscription = &args[1];

    // show available types
    if args.len() == 3 && args[2] == "types" {
        print_available_types();
        return;
    }

    // check if second argument is a search or subsearch
    if args.len() == 3 && (args[2].starts_with("search:") || args[2].starts_with("subsearch:")) {
        let is_subsearch = args[2].starts_with("subsearch:");
        let search_term = if is_subsearch { &args[2][10..] } else { &args[2][7..] };
        
        // get all resources and search across them
        match list_all_resources(subscription).await {
            Ok(data) => {
                let results = if is_subsearch {
                    subsearch_json(&data, search_term)
                } else {
                    search_json(&data, search_term)
                };
                
                if (is_subsearch && results.as_object().map_or(true, |obj| obj.is_empty())) ||
                   (!is_subsearch && results.as_array().map_or(true, |arr| arr.is_empty())) {
                    println!("No resources found containing '{}'", search_term);
                } else {
                    println!("{}", serde_json::to_string_pretty(&results).unwrap());
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
        return;
    }

    // if only subscription provided, list all resources by default
    if args.len() == 2 {
        match list_all_resources(subscription).await {
            Ok(data) => {
                println!("{}", serde_json::to_string_pretty(&data).unwrap());
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
        return;
    }

    // handle specific commands: all, or resource type, or resource group name
    if args.len() == 3 && !args[2].starts_with("search:") && !args[2].starts_with("subsearch:") {
        let command = &args[2];
        
        if command == "all" {
            match list_all_resources(subscription).await {
                Ok(data) => {
                    println!("{}", serde_json::to_string_pretty(&data).unwrap());
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
            return;
        }
        
        let mappings = get_resource_mappings();
        
        // check if it's a known resource type
        if mappings.contains_key(command.to_lowercase().as_str()) {
            match list_resources_by_type(subscription, command).await {
                Ok(data) => {
                    println!("{}", serde_json::to_string_pretty(&data).unwrap());
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
            return;
        } else {
            // treat as resource group name - list all resources in RG
            let rg = command;
            let url = format!(
                "https://management.azure.com/subscriptions/{}/resourceGroups/{}/resources?api-version=2021-04-01",
                subscription, rg
            );
            
            let token = match get_azure_token().await {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Error getting token: {}", e);
                    return;
                }
            };
            
            let client = reqwest::Client::new();
            match client.get(&url).bearer_auth(&token).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<Value>().await {
                            Ok(data) => {
                                println!("{}", serde_json::to_string_pretty(&data).unwrap());
                            }
                            Err(_) => {
                                eprintln!("Error: Failed to parse JSON response");
                            }
                        }
                    } else {
                        eprintln!("Error: Failed to list resources in RG '{}': {}", rg, response.status());
                    }
                }
                Err(_) => {
                    eprintln!("Error: Request failed");
                }
            }
            return;
        }
    }

    // handle: subscription rg resource_type
    if args.len() == 4 {
        let rg = &args[2];
        let resource_type = &args[3];
        
        match list_resources_in_rg(subscription, rg, resource_type).await {
            Ok(data) => {
                println!("{}", serde_json::to_string_pretty(&data).unwrap());
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
        return;
    }

    // need at least subscription, rg, resource_type, resource_name for specific resource queries
    if args.len() < 5 {
        eprintln!("Error: Need subscription, resource-group, resource-type, and resource-name for specific resource queries");
        eprintln!("Usage: {} <subscription> <resource-group> <resource-type> <resource-name> [field|search:term|subsearch:term]", args[0]);
        eprintln!("Or use: {} <subscription> search:term", args[0]);
        eprintln!("Or use: {} <subscription> subsearch:term", args[0]);
        return;
    }

    let rg = &args[2];
    let resource_type = &args[3];
    let resource_name = &args[4];
    let query = args.get(5);

    // get the specific resource data
    let data = match get_resource_json(subscription, rg, resource_type, resource_name).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };

    // handle different query types for specific resource
    match query {
        None => {
            // no query - return full JSON
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
        Some(q) if q.starts_with("search:") => {
            // search query on specific resource - returns full resource
            let search_term = &q[7..]; // remove "search:" prefix
            let results = search_json(&data, search_term);
            if results.as_array().map_or(true, |arr| arr.is_empty()) && !results.is_object() {
                println!("No matches found for '{}'", search_term);
            } else {
                println!("{}", serde_json::to_string_pretty(&results).unwrap());
            }
        }
        Some(q) if q.starts_with("subsearch:") => {
            // subsearch query on specific resource - returns just matching fields
            let search_term = &q[10..]; // remove "subsearch:" prefix
            let results = subsearch_json(&data, search_term);
            if results.as_object().map_or(true, |obj| obj.is_empty()) {
                println!("No matches found for '{}'", search_term);
            } else {
                println!("{}", serde_json::to_string_pretty(&results).unwrap());
            }
        }
        Some(field) => {
            // field query on specific resource
            match get_field(&data, field) {
                Some(value) => println!("{}", serde_json::to_string_pretty(&value).unwrap()),
                None => eprintln!("Field '{}' not found", field),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_field() {
        let data = serde_json::json!({
            "name": "test-resource",
            "location": "eastus",
            "properties": {
                "size": "Standard_B2s"
            }
        });

        assert_eq!(get_field(&data, "name").unwrap(), serde_json::Value::String("test-resource".to_string()));
        assert_eq!(get_field(&data, "location").unwrap(), serde_json::Value::String("eastus".to_string()));
        assert!(get_field(&data, "nonexistent").is_none());
    }

    #[test]
    fn test_search_json() {
        let data = serde_json::json!({
            "name": "test-resource",
            "size": "Standard_B2s",
            "diskSize": "30",
            "location": "eastus",
            "properties": {
                "hardwareProfile": {
                    "size": "Standard_B2s"
                }
            }
        });

        let results = search_json(&data, "size");
        // the search should find the resource since it contains "size" fields
        assert!(!results.as_array().unwrap_or(&vec![]).is_empty() || results.is_object());
    }

    #[test]
    fn test_resource_mappings() {
        let mappings = get_resource_mappings();
        assert!(mappings.contains_key("network"));
        assert!(mappings.contains_key("storage"));
        assert!(mappings.contains_key("vm"));
        assert_eq!(mappings.get("network").unwrap().0, "Microsoft.Network/virtualNetworks");
    }
}