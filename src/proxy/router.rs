use crate::config::schema::{ProxyRoute, RateLimitConfig}; 
use crate::config::schema::BasicAuthConfig; 

#[derive(Debug, Clone)] 
pub struct MatchedRoute { 
    pub prefix: String, 
    pub upstream: String, 
    pub rate_limit: Option<RateLimitConfig>, 
    pub basic_auth: Option<BasicAuthConfig>, 
}

pub fn match_proxy_route( 
                          routes: &[ProxyRoute], 
                          path: &str, 
) -> Option<MatchedRoute> { 
    routes 
        .iter() 
        .filter(|route| path_matches_route(path, &route.path)) 
        .max_by_key(|route| route.path.len()) 
        .map(|route| MatchedRoute { 
            prefix: route.path.clone(), 
            upstream: route.upstream.clone(), 
            rate_limit:route.rate_limit.clone(),
            basic_auth: route.basic_auth.clone(), 
        }) 
}

pub fn strip_route_prefix(path: &str, prefix: &str) -> String { 
    if path == prefix { 
        return "/".to_string(); 
    }

    path.strip_prefix(prefix) 
        .filter(|rest| rest.starts_with('/')) 
        .unwrap_or(path) 
        .to_string() 
}

fn path_matches_route(path: &str, prefix: &str) -> bool { 
    path == prefix || path.starts_with(&format!("{prefix}/")) 
}