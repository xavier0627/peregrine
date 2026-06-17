use std::path::Path; 

pub fn content_type(path: &Path) -> &'static str { 
    match path.extension().and_then(|value| value.to_str()) { 
        Some("html") => "text/html; charset=utf-8", 
        Some("css") => "text/css; charset=utf-8", 
        Some("js") => "application/javascript; charset=utf-8", 
        Some("json") => "application/json", 
        Some("png") => "image/png", 
        Some("jpg") | Some("jpeg") => "image/jpeg", 
        Some("svg") => "image/svg+xml", 
        _ => "application/octet-stream", 
    }
}
