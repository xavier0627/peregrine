use std::path::Path; 

use anyhow::Context; 
use anyhow::Result; 

use crate::config::schema::AppConfig; 

pub fn load_config(path: &Path) -> Result<AppConfig> { 
    let text = std::fs::read_to_string(path) 
        .with_context(|| format!("cannot read config file: {}", path.display()))?; 

    let config: AppConfig = serde_yaml::from_str(&text) 
        .with_context(|| format!("cannot parse config file: {}", path.display()))?; 

    Ok(config) 
}