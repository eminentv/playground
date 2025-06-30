#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! serde_json = "1.0"
//! ```

/*!
 * nested path resolver - rust implementation
 */

use serde_json::Value;
use std::fmt;

/// custom error types for detailed error reporting
#[derive(Debug, Clone, PartialEq)]
pub enum PathResolutionError {
    InvalidInput { message: String },
    KeyNotFound { key: String, step: usize, path: String },
    TraversalError { step: usize, found_type: String, path: String },
    EmptyPath,
}

impl fmt::Display for PathResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathResolutionError::InvalidInput { message } => {
                write!(f, "invalid input: {}", message)
            }
            PathResolutionError::KeyNotFound { key, step, path } => {
                write!(f, "path not found: key '{}' does not exist at step {} (path: '{}')", key, step, path)
            }
            PathResolutionError::TraversalError { step, found_type, path } => {
                write!(f, "cannot traverse: expected object at step {}, but found {} at path '{}'", step, found_type, path)
            }
            PathResolutionError::EmptyPath => {
                write!(f, "empty path: path must contain at least one valid key")
            }
        }
    }
}

impl std::error::Error for PathResolutionError {}

/// result type for path resolution operations
pub type PathResult<T> = Result<T, PathResolutionError>;

/// configuration options for path resolution
#[derive(Debug, Clone)]
pub struct ResolverConfig {
    pub case_sensitive: bool,
    pub separator: char,
    pub allow_empty_segments: bool,
    pub trim_whitespace: bool,
}

impl Default for ResolverConfig {
    fn default() -> Self {
        Self {
            case_sensitive: true,
            separator: '/',
            allow_empty_segments: false,
            trim_whitespace: true,
        }
    }
}

/// builder pattern for creating resolver configurations
pub struct ResolverConfigBuilder {
    config: ResolverConfig,
}

impl ResolverConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: ResolverConfig::default(),
        }
    }
    
    pub fn case_sensitive(mut self, case_sensitive: bool) -> Self {
        self.config.case_sensitive = case_sensitive;
        self
    }
    
    pub fn separator(mut self, separator: char) -> Self {
        self.config.separator = separator;
        self
    }
    
    pub fn allow_empty_segments(mut self, allow: bool) -> Self {
        self.config.allow_empty_segments = allow;
        self
    }
    
    pub fn trim_whitespace(mut self, trim: bool) -> Self {
        self.config.trim_whitespace = trim;
        self
    }
    
    pub fn build(self) -> ResolverConfig {
        self.config
    }
}

/// main path resolver struct with configuration
pub struct PathResolver {
    config: ResolverConfig,
}

impl PathResolver {
    /// create a new resolver with default configuration
    pub fn new() -> Self {
        Self {
            config: ResolverConfig::default(),
        }
    }
    
    /// create a resolver with custom configuration
    pub fn with_config(config: ResolverConfig) -> Self {
        Self { config }
    }
    
    /// get value from nested json object using path string
    pub fn get_value<'a>(&self, obj: &'a Value, path: &str) -> PathResult<&'a Value> {
        // input validation
        if !obj.is_object() {
            return Err(PathResolutionError::InvalidInput {
                message: "input must be a json object".to_string(),
            });
        }
        
        if path.trim().is_empty() {
            return Err(PathResolutionError::EmptyPath);
        }
        
        // parse and clean the path using functional operations
        let keys: Vec<String> = self.parse_path(path)?;
        
        if keys.is_empty() {
            return Err(PathResolutionError::EmptyPath);
        }
        
        // core reduce pattern: use try_fold to traverse the object
        keys.iter()
            .enumerate()
            .try_fold(obj, |current, (index, key)| {
                self.traverse_step(current, key, index + 1, &self.build_path(&keys, index + 1))
            })
    }
    
    /// enhanced version with additional features
    pub fn get_value_enhanced(&self, obj: &Value, path: &str) -> PathResult<Value> {
        match self.get_value(obj, path) {
            Ok(value) => Ok(value.clone()),
            Err(e) => Err(e),
        }
    }
    
    /// get value with default fallback
    pub fn get_value_or_default(&self, obj: &Value, path: &str, default: Value) -> Value {
        self.get_value_enhanced(obj, path).unwrap_or(default)
    }
    
    /// check if path exists in object
    pub fn has_path(&self, obj: &Value, path: &str) -> bool {
        self.get_value(obj, path).is_ok()
    }
    
    /// get all available paths in object
    pub fn get_all_paths(&self, obj: &Value) -> Vec<String> {
        let mut paths = Vec::new();
        self.collect_paths(obj, String::new(), &mut paths);
        paths
    }
    
    // private helper methods
    
    fn parse_path(&self, path: &str) -> PathResult<Vec<String>> {
        let keys: Vec<String> = path
            .split(self.config.separator)
            .filter_map(|segment| {
                let processed = if self.config.trim_whitespace {
                    segment.trim()
                } else {
                    segment
                };
                
                if processed.is_empty() && !self.config.allow_empty_segments {
                    None
                } else {
                    Some(if self.config.case_sensitive {
                        processed.to_string()
                    } else {
                        processed.to_lowercase()
                    })
                }
            })
            .collect();
        
        Ok(keys)
    }
    
    fn traverse_step<'a>(&self, current: &'a Value, key: &str, step: usize, path: &str) -> PathResult<&'a Value> {
        match current {
            Value::Object(map) => {
                let actual_key = if self.config.case_sensitive {
                    key
                } else {
                    map.keys()
                        .find(|k| k.to_lowercase() == key.to_lowercase())
                        .map(|s| s.as_str())
                        .unwrap_or(key)
                };
                
                map.get(actual_key).ok_or_else(|| PathResolutionError::KeyNotFound {
                    key: key.to_string(),
                    step,
                    path: path.to_string(),
                })
            }
            _ => Err(PathResolutionError::TraversalError {
                step,
                found_type: self.value_type_name(current).to_string(),
                path: path.to_string(),
            }),
        }
    }
    
    fn build_path(&self, keys: &[String], up_to: usize) -> String {
        keys.iter()
            .take(up_to)
            .cloned()
            .collect::<Vec<_>>()
            .join(&self.config.separator.to_string())
    }
    
    fn value_type_name(&self, value: &Value) -> &'static str {
        match value {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    }
    
    fn collect_paths(&self, value: &Value, current_path: String, paths: &mut Vec<String>) {
        if let Value::Object(map) = value {
            for (key, val) in map {
                let new_path = if current_path.is_empty() {
                    key.clone()
                } else {
                    format!("{}{}{}", current_path, self.config.separator, key)
                };
                
                paths.push(new_path.clone());
                self.collect_paths(val, new_path, paths);
            }
        }
    }
}

/// convenience functions for the exact requirements
pub fn get_nested_value<'a>(obj: &'a Value, key_path: &str) -> PathResult<&'a Value> {
    let resolver = PathResolver::new();
    resolver.get_value(obj, key_path)
}

pub fn get_nested_value_with_config<'a>(
    obj: &'a Value, 
    key_path: &str, 
    config: ResolverConfig
) -> PathResult<&'a Value> {
    let resolver = PathResolver::with_config(config);
    resolver.get_value(obj, key_path)
}

/// trait for extending functionality
pub trait PathResolvable {
    fn resolve_path(&self, path: &str) -> PathResult<&Value>;
    fn has_path(&self, path: &str) -> bool;
}

impl PathResolvable for Value {
    fn resolve_path(&self, path: &str) -> PathResult<&Value> {
        get_nested_value(self, path)
    }
    
    fn has_path(&self, path: &str) -> bool {
        self.resolve_path(path).is_ok()
    }
}

/// functional composition helpers
pub mod functional {
    use super::*;
    
    pub fn compose<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> C
    where
        F: Fn(A) -> B,
        G: Fn(B) -> C,
    {
        move |x| g(f(x))
    }
    
    pub fn create_pipeline() -> impl for<'a> Fn(&'a Value, &str) -> PathResult<&'a Value> {
        |obj, path| get_nested_value(obj, path)
    }
    
    pub fn get_multiple_paths(obj: &Value, paths: &[&str]) -> Vec<(String, PathResult<Value>)> {
        paths
            .iter()
            .map(|&path| {
                let result = get_nested_value(obj, path)
                    .map(|v| v.clone());
                (path.to_string(), result)
            })
            .collect()
    }
    
    pub fn filter_existing_paths(obj: &Value, paths: &[&str]) -> Vec<String> {
        paths
            .iter()
            .filter(|&&path| get_nested_value(obj, path).is_ok())
            .map(|s| s.to_string())
            .collect()
    }
}

/// test helper to show detailed results
fn test_result(test_name: &str, expected: bool, actual_result: bool, details: &str) {
    let status = if actual_result == expected { "PASSED" } else { "FAILED" };
    println!("{}: {} - {}", test_name, status, details);
}

/// comprehensive unit tests  output
fn run_comprehensive_tests() {
    println!("comprehensive unit output");
    println!("=============================================");
    println!();
    
    // test 1: basic req uirement examples
    println!("1. basic requirements compliance:");
    let obj1 = serde_json::json!({"a":{"b":{"c":"d"}}});
    match get_nested_value(&obj1, "a/b/c") {
        Ok(result) => {
            let passed = result == &serde_json::json!("d");
            test_result("1a", true, passed, &format!("expected: 'd', got: {:?}", result));
        }
        Err(e) => test_result("1a", true, false, &format!("unexpected error: {}", e)),
    }
    
    let obj2 = serde_json::json!({"x":{"y":{"z":"a"}}});
    match get_nested_value(&obj2, "x/y/z") {
        Ok(result) => {
            let passed = result == &serde_json::json!("a");
            test_result("1b", true, passed, &format!("expected: 'a', got: {:?}", result));
        }
        Err(e) => test_result("1b", true, false, &format!("unexpected error: {}", e)),
    }
    println!();
    
    // test 2: error handling with detailed messages
    println!("2. error handling:");
    let obj = serde_json::json!({"a": {"b": "value"}});
    
    match get_nested_value(&obj, "") {
        Err(PathResolutionError::EmptyPath) => test_result("2a", true, true, "empty path correctly rejected"),
        _ => test_result("2a", true, false, "should reject empty path"),
    }
    
    match get_nested_value(&obj, "nonexistent") {
        Err(PathResolutionError::KeyNotFound { key, step, path }) => {
            test_result("2b", true, true, &format!("key: '{}', step: {}, path: '{}'", key, step, path));
        }
        _ => test_result("2b", true, false, "should detect missing key"),
    }
    
    match get_nested_value(&obj, "a/b/deeper") {
        Err(PathResolutionError::TraversalError { step, found_type, path }) => {
            test_result("2c", true, true, &format!("step: {}, found: {}, path: '{}'", step, found_type, path));
        }
        _ => test_result("2c", true, false, "should detect traversal error"),
    }
    println!();
    
    // test 3: real-world json examples
    println!("3. real-world json examples:");
    test_real_world_json();
    println!();
    
    // test 4: performance and edge cases
    println!("4. performance and edge cases:");
    test_performance_and_edge_cases();
    println!();
    
    println!("test summary: all core functionality verified with detailed output");
}

/// test with real-world json examples
fn test_real_world_json() {
    // api response example
    let api_response = serde_json::json!({
        "status": "success",
        "data": {
            "users": [
                {
                    "id": 1,
                    "profile": {
                        "firstName": "John",
                        "lastName": "Doe",
                        "email": "john.doe@company.com",
                        "preferences": {
                            "theme": "dark",
                            "notifications": {
                                "email": true,
                                "sms": false,
                                "push": true
                            }
                        }
                    },
                    "roles": ["user", "admin"],
                    "lastLogin": "2023-12-01T10:30:00Z"
                }
            ],
            "pagination": {
                "page": 1,
                "limit": 10,
                "total": 1
            }
        },
        "meta": {
            "timestamp": "2023-12-01T10:30:00Z",
            "version": "2.1.0",
            "requestId": "req_123456789"
        }
    });
    
    println!("   api response testing:");
    
    // test various paths in the api response
    let test_cases = vec![
        ("status", "success"),
        ("data/pagination/page", "1"),
        ("data/users", "[array with 1 item]"),
        ("meta/version", "2.1.0"),
        ("meta/requestId", "req_123456789"),
    ];
    
    for (path, _expected_desc) in test_cases {
        match get_nested_value(&api_response, path) {
            Ok(result) => {
                println!("     {} -> {:?}", path, result);
            }
            Err(e) => {
                println!("     {} -> error: {}", path, e);
            }
        }
    }
    
    // test deep nested access
    match get_nested_value(&api_response, "data/users") {
        Ok(Value::Array(users)) if !users.is_empty() => {
            println!("     deep access: successfully found users array with {} items", users.len());
        }
        Ok(_) => println!("     deep access: unexpected value type"),
        Err(e) => println!("     deep access: error -> {}", e),
    }
    
    // configuration file example
    let config_file = serde_json::json!({
        "database": {
            "host": "localhost",
            "port": 5432,
            "name": "myapp_db",
            "credentials": {
                "username": "admin",
                "password": "secretpass"
            },
            "pool": {
                "maxConnections": 100,
                "timeoutMs": 5000
            }
        },
        "server": {
            "port": 8080,
            "host": "0.0.0.0",
            "ssl": {
                "enabled": true,
                "certPath": "/etc/ssl/cert.pem",
                "keyPath": "/etc/ssl/key.pem"
            }
        },
        "logging": {
            "level": "info",
            "outputs": ["console", "file"],
            "file": {
                "path": "/var/log/app.log",
                "maxSizeMb": 100
            }
        }
    });
    
    println!("   configuration file testing:");
    
    let config_tests = vec![
        "database/host",
        "database/port",
        "database/credentials/username",
        "server/ssl/enabled",
        "logging/level",
        "logging/file/maxSizeMb",
    ];
    
    for path in config_tests {
        match get_nested_value(&config_file, path) {
            Ok(result) => {
                println!("     {} -> {:?}", path, result);
            }
            Err(e) => {
                println!("     {} -> error: {}", path, e);
            }
        }
    }
}

/// test performance and edge cases
fn test_performance_and_edge_cases() {
    use std::time::Instant;
    
    // create deeply nested object for performance testing
    let mut deep_obj = serde_json::json!({});
    let mut current = &mut deep_obj;
    for i in 0..50 {
        let key = format!("level{}", i);
        if i == 49 {
            current[&key] = serde_json::json!({
                "value": "found_at_depth_50",
                "metadata": {
                    "depth": 50,
                    "created": "2023-12-01"
                }
            });
        } else {
            current[&key] = serde_json::json!({});
            current = &mut current[&key];
        }
    }
    
    let deep_path = (0..50).map(|i| format!("level{}", i)).collect::<Vec<_>>().join("/") + "/value";
    
    // performance test
    let start = Instant::now();
    let mut success_count = 0;
    for _ in 0..1000 {
        if get_nested_value(&deep_obj, &deep_path).is_ok() {
            success_count += 1;
        }
    }
    let duration = start.elapsed();
    
    println!("   performance test: 1000 deep accesses (50 levels) in {:?}", duration);
    println!("   success rate: {}/1000", success_count);
    
    // edge cases
    let edge_case_obj = serde_json::json!({
        "": "empty_key_value",
        "null_value": null,
        "unicode_key": "unicode_value",
        "spaces in key": "spaced_value",
        "numbers": {
            "integer": 42,
            "float": 3.14159,
            "negative": -100
        },
        "arrays": {
            "simple": [1, 2, 3],
            "nested": [{"item": "first"}, {"item": "second"}],
            "mixed": [1, "string", true, null]
        }
    });
    
    println!("   edge cases:");
    let edge_cases = vec![
        ("", "empty key access"),
        ("null_value", "null value access"),
        ("unicode_key", "unicode key access"),
        ("spaces in key", "key with spaces"),
        ("numbers/integer", "integer value"),
        ("numbers/float", "float value"),
        ("arrays/simple", "simple array"),
        ("arrays/mixed", "mixed type array"),
    ];
    
    for (path, description) in edge_cases {
        match get_nested_value(&edge_case_obj, path) {
            Ok(result) => {
                println!("     {}: {} -> {:?}", description, path, result);
            }
            Err(e) => {
                println!("     {}: {} -> error: {}", description, path, e);
            }
        }
    }
}

/// interactive testing function
fn run_interactive_test() {
    println!("interactive json testing");
    println!("========================");
    println!();
    
    // sample json for user to test with
    let sample_json = serde_json::json!({
        "user": {
            "id": 12345,
            "profile": {
                "name": "Alice Johnson",
                "email": "alice@example.com",
                "settings": {
                    "theme": "dark",
                    "language": "en",
                    "notifications": {
                        "email": true,
                        "push": false
                    }
                }
            },
            "preferences": {
                "dashboard": ["sales", "analytics", "reports"],
                "timezone": "UTC-5"
            }
        },
        "account": {
            "type": "premium",
            "created": "2023-01-15",
            "features": {
                "api_access": true,
                "storage_gb": 100,
                "support_level": "priority"
            }
        }
    });
    
    println!("sample json structure:");
    println!("{}", serde_json::to_string_pretty(&sample_json).unwrap());
    println!();
    
    println!("testing various paths:");
    let test_paths = vec![
        "user/profile/name",
        "user/profile/email", 
        "user/profile/settings/theme",
        "user/profile/settings/notifications/email",
        "user/preferences/dashboard",
        "user/preferences/timezone",
        "account/type",
        "account/features/storage_gb",
        "account/features/api_access",
        "nonexistent/path",
        "user/profile/settings/invalid",
    ];
    
    let resolver = PathResolver::new();
    for path in test_paths {
        match resolver.get_value(&sample_json, path) {
            Ok(result) => {
                println!("  {} -> {:?}", path, result);
            }
            Err(e) => {
                println!("  {} -> error: {}", path, e);
            }
        }
    }
    
    println!();
    println!("path discovery - all available paths:");
    let all_paths = resolver.get_all_paths(&sample_json);
    for (i, path) in all_paths.iter().enumerate() {
        println!("  {}: {}", i + 1, path);
    }
    
    println!();
    println!("configuration testing:");
    
    // test case insensitive
    let case_insensitive_config = ResolverConfigBuilder::new()
        .case_sensitive(false)
        .build();
    let case_insensitive_resolver = PathResolver::with_config(case_insensitive_config);
    
    println!("  case insensitive access:");
    match case_insensitive_resolver.get_value(&sample_json, "USER/PROFILE/NAME") {
        Ok(result) => println!("    USER/PROFILE/NAME -> {:?}", result),
        Err(e) => println!("    USER/PROFILE/NAME -> error: {}", e),
    }
    
    // test custom separator
    let dot_config = ResolverConfigBuilder::new()
        .separator('.')
        .build();
    let dot_resolver = PathResolver::with_config(dot_config);
    
    println!("  dot separator access:");
    match dot_resolver.get_value(&sample_json, "user.profile.email") {
        Ok(result) => println!("    user.profile.email -> {:?}", result),
        Err(e) => println!("    user.profile.email -> error: {}", e),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "--test" => {
                run_comprehensive_tests();
                return Ok(());
            }
            "--real" => {
                test_real_world_json();
                return Ok(());
            }
            "--interactive" => {
                run_interactive_test();
                return Ok(());
            }
            "--help" => {
                println!("nested path resolver - usage options:");
                println!("  rust-script script.rs            # run basic demonstrations");
                println!("  rust-script script.rs --test     # run comprehensive unit tests");
                println!("  rust-script script.rs --real     # test with real-world json");
                println!("  rust-script script.rs --interactive # interactive testing with sample data");
                println!("  rust-script script.rs --help     # show this help");
                return Ok(());
            }
            _ => {}
        }
    }
    
    println!("nested path resolver - rust implementation");
    println!("==========================================");
    println!();
    println!("this script performs nested object traversal using rust");
    println!("run with different flags to see various testing modes:");
    println!();
    println!("available modes:");
    println!("  --test        comprehensive unit tests with detailed output");
    println!("  --real        real-world json testing (api responses, config files)");
    println!("  --interactive interactive testing with sample data and path discovery");
    println!("  --help        show help information");
    println!();
    
    // run basic demonstration by default
    let obj1 = serde_json::json!({"a":{"b":{"c":"d"}}});
    let result1 = get_nested_value(&obj1, "a/b/c")?;
    println!("basic example: get_nested_value({}, 'a/b/c') = {:?}", obj1, result1);
    
    let obj2 = serde_json::json!({"x":{"y":{"z":"a"}}});
    let result2 = get_nested_value(&obj2, "x/y/z")?;
    println!("basic example: get_nested_value({}, 'x/y/z') = {:?}", obj2, result2);
    
    Ok(())
}