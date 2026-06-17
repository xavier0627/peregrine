use indexmap::IndexMap; 

use crate::config::schema::AppConfig; 
use crate::filter::cors::apply_cors_headers; 
use crate::filter::security_headers::apply_security_headers; 

pub fn response_headers(config: &AppConfig) -> IndexMap<String, String> { 
    let mut headers = config.headers.clone(); 

    apply_security_headers(&mut headers, config.security_headers); 

    apply_cors_headers(&mut headers, config.cors.as_ref()); 

    headers 
}