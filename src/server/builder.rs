use pingora_core::server::configuration::Opt; 
use pingora_core::server::Server; 
use pingora_proxy::http_proxy_service; 

use crate::proxy::service::Peregrine; 
use crate::config::schema::AppConfig; 

pub fn run_server(config: AppConfig) { 
    let listen = config.listen.clone(); 

    let opt = Opt { 
        upgrade: false, 
        daemon: false, 
        nocapture: false, 
        test: false, 
        conf: None, 
    };

    let mut pg_server = Server::new(Some(opt)).expect("failed to create pingora server"); 

    pg_server.bootstrap(); 

    let proxy = Peregrine::new(config); 

    let mut service = http_proxy_service(&pg_server.configuration, proxy); 

    service.add_tcp(&listen); 

    pg_server.add_service(service); 

    println!("Peregrine listening on http://{listen}"); 

    pg_server.run_forever(); 
}