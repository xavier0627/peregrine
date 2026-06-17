use anyhow::Result; 
use pingora_core::upstreams::peer::HttpPeer; 
use url::Url; 


pub fn build_http_peer(upstream: &str) -> Result<HttpPeer> { 
    let url = Url::parse(upstream)?; 

    let host = url.host_str().ok_or_else(|| anyhow::anyhow!("upstream missing host"))?; 

    let port = url.port_or_known_default().ok_or_else(|| anyhow::anyhow!("upstream missing port"))?; 

    let addr = format!("{host}:{port}"); 

    let use_tls = url.scheme() == "https"; 

    let sni = if use_tls { host.to_string() } else { String::new() }; 

    Ok(HttpPeer::new(addr, use_tls, sni)) 
}

pub fn upstream_authority(upstream: &str) -> Result<String> { 
    let url = Url::parse(upstream)?; 

    let host = url.host_str().ok_or_else(|| anyhow::anyhow!("upstream missing host"))?; 

    let Some(port) = url.port() else { 
        return Ok(host.to_string()); 
    };

    Ok(format!("{host}:{port}")) 
}