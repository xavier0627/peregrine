use serde::Deserialize; 
use indexmap::IndexMap; 

#[derive(Debug, Clone, Deserialize)] 
#[serde(rename_all = "camelCase")] 
pub struct AppConfig { 
    #[serde(default = "default_listen")] 
    pub listen: String, 
    #[serde(default = "default_health_path")] 
    pub health_path: String, 
    #[serde(default = "default_static_dir")] 
    pub static_dir: String, 
    #[serde(default)] 
    pub proxy: Vec<ProxyRoute>, 
    #[serde(default = "default_log_format")] 
    pub log_format: String, 
    #[serde(default)] 
    pub headers: IndexMap<String, String>, 
    #[serde(default)] 
    pub security_headers: bool, 
    #[serde(default)] 
    pub cors: Option<CorsConfig>, 
    pub rate_limit: Option<RateLimitConfig>, 

}

#[derive(Debug, Clone, Deserialize)] 
pub struct ProxyRoute { 
    pub path: String, 
    pub upstream: String, 

    #[serde(rename = "rateLimit")] 
    pub rate_limit: Option<RateLimitConfig>, 

    #[serde(rename = "basicAuth")] 
    pub basic_auth: Option<BasicAuthConfig>, 

}

#[derive(Debug, Clone, Deserialize)] 
#[serde(rename_all = "camelCase")] 
pub struct CorsConfig { 
    pub allow_origin: Option<String>, 
    pub allow_methods: Option<String>, 
    pub allow_headers: Option<String>, 
}

#[derive(Debug, Clone, Deserialize)] 
#[serde(rename_all = "camelCase")] 
pub struct RateLimitConfig { 
    pub requests: u64, 

    pub window_seconds: u64, 
}

#[derive(Debug, Clone, Deserialize)] 
pub struct BasicAuthConfig { 
    pub username: String, 
    pub password: String, 
}

fn default_listen() -> String { 
    "0.0.0.0:8080".to_string() 
}

fn default_health_path() -> String { 
    "/__health__".to_string() 
}

fn default_static_dir() -> String { 
    "./dist".to_string() 
}
fn default_log_format() -> String { 
    "text".to_string() 
}