use async_trait::async_trait; 
use pingora_core::upstreams::peer::HttpPeer; 
use pingora_core::Result; 
use pingora_proxy::{ProxyHttp, Session}; 
use pingora_http::{RequestHeader, ResponseHeader}; 
use bytes::Bytes;
use indexmap::IndexMap;


use crate::config::schema::AppConfig;
use crate::handler::health::handle_health; 
use crate::handler::matchs::handle_not_found; 
use crate::handler::static_files::handle_static; 
use crate::proxy::router::match_proxy_route; 
use crate::proxy::upstream::build_http_peer; 
use crate::proxy::upstream::upstream_authority; 
use crate::proxy::router::strip_route_prefix; 
use crate::proxy::ctx::RequestContext; 
use crate::handler::response::insert_config_headers; 
use crate::filter::cors::is_preflight; 
use crate::filter::response_headers::response_headers; 
use crate::handler::response::write_empty_response; 
use crate::filter::rate_limit::{RateLimitDecision, RateLimiter}; 
use crate::handler::response::write_response; 
use crate::filter::auth::check_basic_auth; 



pub struct Peregrine{ 
    config: AppConfig, 
    rate_limiter: RateLimiter, 
}

impl Peregrine { 
    pub fn new(config: AppConfig) -> Self { 
        let rate_limiter = RateLimiter::new(); 

        Self { 
            config, 
            rate_limiter, 
        }
    }
}

#[async_trait] 
impl ProxyHttp for Peregrine { 
    type CTX = RequestContext; 

    fn new_ctx(&self) -> Self::CTX { 
        RequestContext::new() 
    }

    async fn request_filter( 
                             &self, 
                             session: &mut Session, 
                             ctx: &mut Self::CTX, 
    ) -> Result<bool> { 
        let mut headers = response_headers(&self.config); 
        if is_preflight(session) { 
            write_empty_response(session, 204, &headers).await?; 
            return Ok(true); 
        }
        let path = session.req_header().uri.path().to_string(); 
        if path == self.config.health_path { 
            handle_health(session, &headers).await?; 
            return Ok(true); 
        }
        let client_ip = session 
            .client_addr() 
            .and_then(|addr| addr.as_inet()) 
            .map(|addr| addr.ip().to_string()) 
            .unwrap_or_else(|| "unknown".to_string()); 

        if let Some(route) = match_proxy_route(&self.config.proxy, &path) { 
            if !check_basic_auth(session.req_header(), route.basic_auth.as_ref()) { 
                headers.insert("www-authenticate".to_string(),"Basic Required".to_string());
                write_response(session,401, "text/plain",Bytes::from_static(b"Unauthorized\n"),&headers).await?;
                return Ok(true); 
            } 

            
            let bucket = format!("route:{}", route.prefix); 
            let limit_config = route.rate_limit.as_ref().or(self.config.rate_limit.as_ref()); 
            let decision = self.rate_limiter.check(&client_ip,&bucket,  limit_config); 
            ctx.rate_limit_decision = Some(decision.clone()); 
            if !decision.allowed { 
                ctx.rate_limited = true; 
                handle_too_many_requests(session, &headers, &decision).await?; 
                return Ok(true); 
            } 

            ctx.route = Some(route); 
            return Ok(false); 
        }

        if handle_static(session, &self.config.static_dir,&headers).await? { 
            return Ok(true); 
        }

        handle_not_found(session,&headers).await?; 

        Ok(true) 
    }
    
    async fn upstream_peer( 
                            &self, 
                            _session: &mut Session, 
                            ctx: &mut Self::CTX, 
    ) -> Result<Box<HttpPeer>> { 
        let upstream = ctx.route.as_ref().map(|route| route.upstream.as_str()).unwrap_or("http://127.0.0.1:1"); 
        let peer = build_http_peer(upstream).map_err(|err| { 
            pingora_core::Error::explain( 
                                          pingora_core::ErrorType::InternalError, 
                                          format!("invalid upstream {upstream}: {err}"), 
            )
        })?; 
        Ok(Box::new(peer)) 
    }

    async fn upstream_request_filter( 
                                      &self, 
                                      session: &mut Session, 
                                      upstream_request: &mut RequestHeader, 
                                      ctx: &mut Self::CTX, 
    ) -> Result<()> { 
        add_forwarded_headers(session, upstream_request)?; 

        if let Some(route) = ctx.route.as_ref() { 
            rewrite_host_header(upstream_request, &route.upstream)?; 
            rewrite_path(upstream_request, &route.prefix)?; 
        }
        Ok(()) 
    }
    async fn response_filter( 
                              &self, 
                              _session: &mut Session, 
                              upstream_response: &mut ResponseHeader, 
                              ctx: &mut Self::CTX, 
    ) -> Result<()> { 
        let headers = response_headers(&self.config); 
        insert_config_headers(upstream_response, &headers)?; 

        if let Some(decision) = ctx.rate_limit_decision.as_ref() { 
            if decision.allowed { 
                insert_rate_limit_headers(upstream_response, decision)?; 
            } 
        }
        Ok(()) 
    }
    async fn logging( 
                      &self, 
                      session: &mut Session, 
                      error: Option<&pingora_core::Error>, 
                      ctx: &mut Self::CTX, 
    ) where Self::CTX: Send + Sync { 
        let method = session.req_header().method.as_str(); 

        let path = session.req_header().uri.path(); 

        let status = session.response_written().map(|response| response.status.as_u16()).unwrap_or(0); 

        let elapsed_ms = ctx.start.elapsed().as_millis(); 

        let upstream = ctx.route.as_ref().map(|route| route.upstream.as_str()).unwrap_or("-"); 

        if let Some(error) = error { 
            tracing::error!( 
              method = method, 
              path = path, 
              status = status, 
              elapsed_ms = elapsed_ms, 
              upstream = upstream, 
              error = %error, 
                limited = ctx.rate_limited, 
              "request failed" 
          ); 
        } else { 
            tracing::info!( 
              method = method, 
              path = path, 
              status = status, 
              elapsed_ms = elapsed_ms, 
              upstream = upstream, 
                limited = ctx.rate_limited, 
              "request completed" 
          ); 
        }
    }
}

fn rewrite_host_header( 
                        upstream_request: &mut RequestHeader, 
                        upstream: &str, 
) -> Result<()> { 
    let host = upstream_authority(upstream).map_err(|err| { 
        pingora_core::Error::explain( 
                                      pingora_core::ErrorType::InternalError, 
                                      format!("invalid upstream host {upstream}: {err}"), 
        )
    })?; 

    upstream_request.insert_header("host", host)?; 

    Ok(()) 
}


fn rewrite_path( 
                 upstream_request: &mut RequestHeader, 
                 prefix: &str, 
) -> Result<()> { 
    let original_path = upstream_request.uri.path().to_string(); 

    let new_path = strip_route_prefix(&original_path, prefix); 

    let new_uri = if let Some(query) = upstream_request.uri.query() { 
        format!("{new_path}?{query}") 
    } else { 
        new_path 
    };

    upstream_request.set_raw_path(new_uri.as_bytes())?; 

    Ok(()) 
}

fn add_forwarded_headers( 
                          session: &Session, 
                          upstream_request: &mut RequestHeader, 
) -> Result<()> { 
    let client_ip = session.client_addr().and_then(|addr| addr.as_inet()).map(|addr| addr.ip().to_string()).unwrap_or_else(|| "unknown".to_string()); 
    let x_forwarded_for = match upstream_request.headers.get("x-forwarded-for").and_then(|value| value.to_str().ok()) { 
        Some(existing) => format!("{existing}, {client_ip}"), 
        None => client_ip, 
    };

    upstream_request.insert_header("x-forwarded-for", x_forwarded_for)?; 

    upstream_request.insert_header("x-forwarded-proto", "http")?; 

    if let Some(host) = session.req_header().headers.get("host").and_then(|value| value.to_str().ok()) { 
        upstream_request.insert_header("x-forwarded-host", host)?; 
    }

    let via = match upstream_request.headers.get("via").and_then(|value| value.to_str().ok()) { 
        Some(existing) => format!("{existing}, peregrine"), 
        None => "peregrine".to_string(), 
    };

    upstream_request.insert_header("via", via)?; 

    Ok(()) 
}

async fn handle_too_many_requests(session: &mut Session, headers: &IndexMap<String, String>, decision: &RateLimitDecision) -> Result<()> { 
    let mut headers = headers.clone(); 

    if let Some(retry_after_secs) = decision.retry_after_secs { 
        headers.insert("retry-after".to_string(), retry_after_secs.to_string()); 
    }
    if let Some(limit) = decision.limit {
        headers.insert("x-ratelimit-limit".to_string(), limit.to_string());
    }
    if let Some(remaining) = decision.remaining {
        headers.insert("x-ratelimit-remaining".to_string(), remaining.to_string());
    }
    if let Some(reset_after_secs) = decision.reset_after_secs {
        headers.insert("x-ratelimit-reset".to_string(), reset_after_secs.to_string());
    }

    write_response( 
        session, 
        429, 
        "text/plain", 
        Bytes::from_static(b"too many requests\n"), 
        &headers,
    ).await 
}

fn insert_rate_limit_headers( 
                              response: &mut pingora_http::ResponseHeader, 
                              decision: &crate::filter::rate_limit::RateLimitDecision, 
) -> Result<()> { 
    if let Some(limit) = decision.limit { 
        response.insert_header("x-ratelimit-limit", limit.to_string())?; 
    } 

    if let Some(remaining) = decision.remaining { 
        response.insert_header("x-ratelimit-remaining", remaining.to_string())?; 
    } 

    if let Some(reset) = decision.reset_after_secs { 
        response.insert_header("x-ratelimit-reset", reset.to_string())?; 
    } 

    Ok(()) 
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrite_path_strips_prefix_and_keeps_query() {
        let mut request = RequestHeader::build("GET", b"/api/users?page=1", None).unwrap();

        rewrite_path(&mut request, "/api").unwrap();

        assert_eq!(request.uri.path(), "/users");
        assert_eq!(request.uri.query(), Some("page=1"));
    }

    #[test]
    fn rewrite_path_maps_exact_prefix_to_root() {
        let mut request = RequestHeader::build("GET", b"/api", None).unwrap();

        rewrite_path(&mut request, "/api").unwrap();

        assert_eq!(request.uri.path(), "/");
        assert_eq!(request.uri.query(), None);
    }
}
