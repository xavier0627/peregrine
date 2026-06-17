use base64::engine::general_purpose::STANDARD; 
use base64::Engine; 
use pingora_http::RequestHeader; 

use crate::config::schema::BasicAuthConfig; 

pub fn check_basic_auth( 
                         request: &RequestHeader, 
                         config: Option<&BasicAuthConfig>, 
) -> bool { 
    let Some(config) = config else { 
        return true; 
    }; 

    let Some(value) = request.headers.get("authorization") else { 
        return false; 
    }; 

    let Ok(value) = value.to_str() else { 
        return false; 
    }; 

    let Some(encoded) = value.strip_prefix("Basic ") else { 
        return false; 
    }; 

    let Ok(decoded) = STANDARD.decode(encoded) else { 
        return false; 
    }; 

    let Ok(decoded) = String::from_utf8(decoded) else { 
        return false; 
    }; 

    decoded == format!("{}:{}", config.username, config.password) 
}
