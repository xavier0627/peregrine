use anyhow::Result; 
use url::Url; 
use http::header::HeaderName; 
use http::header::HeaderValue; 
use http::Method; 

use crate::config::schema::AppConfig; 
use crate::config::schema::CorsConfig; 
use crate::config::schema::RateLimitConfig; 
use crate::config::schema::BasicAuthConfig; 


pub fn validate_config(config: &AppConfig) -> Result<()> { 
    validate_health_path(&config.health_path)?; 
    validate_log_format(&config.log_format)?; 
    validate_headers(config)?; 
    validate_cors(config.cors.as_ref())?; 
    validate_rate_limit(config.rate_limit.as_ref())?; 

    validate_proxy_routes(config)?; 

    Ok(()) 
}

fn validate_rate_limit(rate_limit: Option<&RateLimitConfig>) -> Result<()> { 
    let Some(rate_limit) = rate_limit else { 
        return Ok(()); 
    };

    if rate_limit.requests == 0 { 
        anyhow::bail!("rateLimit.requests must be greater than 0"); 
    }

    if rate_limit.window_seconds == 0 { 
        anyhow::bail!("rateLimit.windowSeconds must be greater than 0"); 
    }

    Ok(()) 
}
fn validate_health_path(path: &str) -> Result<()> { 
    if !path.starts_with('/') { 
        anyhow::bail!("healthPath must start with '/'"); 
    }

    Ok(()) 
}
fn validate_cors(cors: Option<&CorsConfig>) -> Result<()> { 
    let Some(cors) = cors else { 
        return Ok(()); 
    };

    if let Some(origin) = &cors.allow_origin { 
        validate_cors_origin(origin)?; 
    }

    if let Some(methods) = &cors.allow_methods { 
        validate_cors_methods(methods)?; 
    }

    if let Some(headers) = &cors.allow_headers { 
        validate_cors_headers(headers)?; 
    }

    Ok(()) 
}


fn validate_cors_origin(origin: &str) -> Result<()> { 
    if origin.trim().is_empty() { 
        anyhow::bail!("cors.allowOrigin cannot be empty"); 
    }

    if origin == "*" { 
        return Ok(()); 
    }

    let url = Url::parse(origin) 
        .map_err(|_| anyhow::anyhow!("cors.allowOrigin must be '*' or a valid http/https origin"))?; 

    match url.scheme() { 
        "http" | "https" => Ok(()), 
        other => anyhow::bail!("cors.allowOrigin scheme must be http or https, got '{other}'"), 
    }
}

fn validate_cors_methods(methods: &str) -> Result<()> { 
    if methods.trim().is_empty() { 
        anyhow::bail!("cors.allowMethods cannot be empty"); 
    }

    for method in methods.split(',') { 
        let method = method.trim(); 

        if method.is_empty() { 
            anyhow::bail!("cors.allowMethods contains an empty method"); 
        }

        Method::from_bytes(method.as_bytes()) 
            .map_err(|_| anyhow::anyhow!("invalid CORS method '{method}'"))?; 
    }

    Ok(()) 
}

fn validate_cors_headers(headers: &str) -> Result<()> { 
    if headers.trim().is_empty() { 
        anyhow::bail!("cors.allowHeaders cannot be empty"); 
    }

    for header in headers.split(',') { 
        let header = header.trim(); 

        if header.is_empty() { 
            anyhow::bail!("cors.allowHeaders contains an empty header name"); 
        }

        HeaderName::from_bytes(header.as_bytes()) 
            .map_err(|_| anyhow::anyhow!("invalid CORS header name '{header}'"))?; 
    }

    Ok(()) 
}

fn validate_headers(config: &AppConfig) -> Result<()> { 
    for (name, value) in &config.headers { 
        validate_header_name(name)?; 
        validate_header_value(name, value)?; 
    }

    Ok(()) 
}

fn validate_header_name(name: &str) -> Result<()> { 
    HeaderName::from_bytes(name.as_bytes()) 
        .map_err(|_| anyhow::anyhow!("invalid response header name '{name}'"))?; 

    let lower = name.to_ascii_lowercase(); 

    if lower == "content-length" { 
        anyhow::bail!("response header 'content-length' cannot be configured manually"); 
    }

    Ok(()) 
}

fn validate_header_value(name: &str, value: &str) -> Result<()> { 
    HeaderValue::from_str(value) 
        .map_err(|_| anyhow::anyhow!("invalid value for response header '{name}'"))?; 

    Ok(()) 
}

fn validate_proxy_routes(config: &AppConfig) -> Result<()> { 
    let mut seen_paths = std::collections::HashSet::new(); 

    for route in &config.proxy { 
        validate_proxy_prefix(&route.path)?; 
        validate_upstream_url(&route.upstream)?; 
        validate_rate_limit(route.rate_limit.as_ref())?; 
        validate_basic_auth(route.basic_auth.as_ref())?; 

        if !seen_paths.insert(route.path.clone()) { 
            anyhow::bail!("duplicate proxy path '{}'", route.path); 
        }
    }

    Ok(()) 
}

fn validate_proxy_prefix(prefix: &str) -> Result<()> { 
    if !prefix.starts_with('/') { 
        anyhow::bail!("proxy route '{prefix}' must start with '/'"); 
    }

    Ok(()) 
}

fn validate_upstream_url(upstream: &str) -> Result<()> { 
    let url = Url::parse(upstream)?; 

    match url.scheme() { 
        "http" | "https" => Ok(()), 
        other => anyhow::bail!("unsupported upstream scheme '{other}' in '{upstream}'"), 
    }
}


fn validate_log_format(format: &str) -> Result<()> { 
    match format { 
        "text" | "json" => Ok(()), 
        other => anyhow::bail!("logFormat must be 'text' or 'json', got '{other}'"), 
    }
}
fn validate_basic_auth(auth: Option<&BasicAuthConfig>) -> Result<()> { 
    let Some(auth) = auth else { 
        return Ok(()); 
    }; 

    if auth.username.trim().is_empty() { 
        anyhow::bail!("basicAuth.username cannot be empty"); 
    } 

    if auth.password.is_empty() { 
        anyhow::bail!("basicAuth.password cannot be empty"); 
    } 

    Ok(()) 
}