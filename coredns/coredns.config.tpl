.:53 {
    # Error logging
    errors
    
    # Health check endpoint
    health :8080
    
    # Ready check endpoint  
    ready :8181
    
    # Kubernetes plugin (for cluster.local domains)
    kubernetes cluster.local in-addr.arpa ip6.arpa {
        pods insecure
        fallthrough in-addr.arpa ip6.arpa
        ttl 30
    }
    
    # Prometheus metrics
    prometheus :9153
    
    # Forward all other queries to upstream DNS servers
    forward . ${upstream_servers} {
        policy round_robin
        health_check 5s
    }
    
    # Cache responses
    cache 30 {
        success 9984 30
        denial 9984 5
    }
    
    # Detect forwarding loops
    loop
    
    # Enable configuration reload
    reload
    
    # Load balance A, AAAA and MX records
    loadbalance
}