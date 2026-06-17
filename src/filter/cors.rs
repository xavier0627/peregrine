use indexmap::IndexMap; 
use pingora_proxy::Session; 

use crate::config::schema::CorsConfig; 

pub fn apply_cors_headers( 
                           headers: &mut IndexMap<String, String>, 
                           cors: Option<&CorsConfig>, 
) { 
    let Some(cors) = cors else { 
        return; 
    };

    let allow_origin = cors.allow_origin.as_deref().unwrap_or("*"); 

    let allow_methods = cors.allow_methods.as_deref().unwrap_or("GET,POST,OPTIONS"); 

    let allow_headers = cors.allow_headers.as_deref().unwrap_or("content-type,authorization"); 

    headers.insert("access-control-allow-origin".to_string(), allow_origin.to_string()); 

    headers.insert("access-control-allow-methods".to_string(), allow_methods.to_string()); 

    headers.insert("access-control-allow-headers".to_string(), allow_headers.to_string()); 
}

pub fn is_preflight(session: &Session) -> bool { 
    let method = session.req_header().method.as_str(); 

    if method != "OPTIONS" { 
        return false; 
    }

    let headers = &session.req_header().headers; 

    headers.contains_key("origin") && headers.contains_key("access-control-request-method") 
}