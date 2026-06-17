use std::path::PathBuf; 

use bytes::Bytes; 
use pingora_core::Result; 
use pingora_proxy::Session; 
use indexmap::IndexMap; 

use crate::handler::response::write_response; 
use crate::util::mime::content_type; 

pub async fn handle_static(session: &mut Session, static_dir: &str,    headers: &IndexMap<String, String>) -> Result<bool> { 
    let request_path = session.req_header().uri.path(); 

    let relative_path = normalize_path(request_path); 

    let Some(relative_path) = relative_path else { 
        return Ok(false); 
    };

    let file_path = PathBuf::from(static_dir).join(relative_path); 

    let bytes = match tokio::fs::read(&file_path).await { 
        Ok(bytes) => bytes, 
        Err(_) => return Ok(false), 
    };

    let body = Bytes::from(bytes); 

    let content_type = content_type(&file_path); 

    write_response(session, 200, content_type, body, headers).await?; 

    Ok(true) 
}

fn normalize_path(path: &str) -> Option<String> { 
    if path.contains("..") { 
        return None; 
    }

    if path == "/" { 
        return Some("index.html".to_string()); 
    }

    let path = path.trim_start_matches('/'); 

    if path.is_empty() { 
        Some("index.html".to_string()) 
    } else { 
        Some(path.to_string()) 
    }
}