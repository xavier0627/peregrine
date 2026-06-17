use bytes::Bytes; 
use pingora_core::Result; 
use pingora_proxy::Session; 

use crate::handler::response::write_response; 

use indexmap::IndexMap; 

pub async fn handle_not_found( 
                               session: &mut Session, 
                               headers: &IndexMap<String, String>, 
) -> Result<()> { 
    let body = Bytes::from_static(b"Not Found"); 

    write_response(session, 404, "text/plain", body, headers).await 
}