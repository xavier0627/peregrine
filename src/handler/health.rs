use bytes::Bytes; 
use pingora_core::Result; 
use pingora_proxy::Session; 
use indexmap::IndexMap; 

use crate::handler::response::write_response; 

pub async fn handle_health( 
                            session: &mut Session, 
                            headers: &IndexMap<String, String>, 
) -> Result<()> { 
    let body = Bytes::from_static(b"{\"status\":\"ok\"}"); 

    write_response(session, 200, "application/json", body, headers).await 
}
