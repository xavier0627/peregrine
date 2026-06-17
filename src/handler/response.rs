use bytes::Bytes; 
use indexmap::IndexMap; 
use pingora_core::Result; 
use pingora_http::ResponseHeader; 
use pingora_proxy::Session; 

pub async fn write_response( 
                             session: &mut Session, 
                             status: u16, 
                             content_type: &str, 
                             body: Bytes, 
                             headers: &IndexMap<String, String>, 
) -> Result<()> { 
    let mut resp = ResponseHeader::build(status, Some(2 + headers.len()))?; 

    resp.insert_header("content-type", content_type)?; 

    resp.insert_header("content-length", body.len().to_string())?; 

    insert_config_headers(&mut resp, headers)?; 

    session.write_response_header(Box::new(resp), false).await?; 

    session.write_response_body(Some(body), true).await 
}

pub fn insert_config_headers( 
                              resp: &mut ResponseHeader, 
                              headers: &IndexMap<String, String>, 
) -> Result<()> { 
    for (name, value) in headers { 
        resp.insert_header(name.clone(), value.as_str())?; 
    }

    Ok(()) 
}

pub async fn write_empty_response( 
                                   session: &mut Session, 
                                   status: u16, 
                                   headers: &IndexMap<String, String>, 
) -> Result<()> { 
    let mut resp = ResponseHeader::build(status, Some(1 + headers.len()))?; 

    resp.insert_header("content-length", "0")?; 

    insert_config_headers(&mut resp, headers)?; 

    session.write_response_header(Box::new(resp), true).await 
}