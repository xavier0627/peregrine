pub fn init(format: &str) { 
    let filter = tracing_subscriber::EnvFilter::try_from_default_env() 
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")); 

    match format { 
        "json" => tracing_subscriber::fmt() 
            .json() 
            .with_env_filter(filter) 
            .init(), 

        _ => tracing_subscriber::fmt() 
            .with_env_filter(filter) 
            .init(), 
    }
}