pub mod args; 
pub mod init; 

use std::path::Path; 

use anyhow::Result; 

use crate::cli::args::Cli; 
use crate::cli::args::Command; 

pub fn dispatch(cli: Cli) -> Result<()> { 
    match cli.command { 
        Some(Command::Init) => run_init_command(&cli), 
        Some(Command::Validate) => run_validate_command(&cli), 
        None => run_serve_command(&cli), 
    }
}

fn run_init_command(cli: &Cli) -> Result<()> { 
    crate::util::logging::init("text"); 
    init::run_init(Path::new(&cli.config)) 
}

fn run_validate_command(cli: &Cli) -> Result<()> { 
    let config = load_config(cli)?; 

    crate::config::validate::validate_config(&config)?; 
    crate::util::logging::init(&config.log_format); 

    tracing::info!("config ok"); 

    Ok(()) 
}

fn run_serve_command(cli: &Cli) -> Result<()> { 
    let config = load_config(cli)?; 

    crate::config::validate::validate_config(&config)?; 
    crate::util::logging::init(&config.log_format); 

    crate::server::builder::run_server(config); 

    Ok(()) 
}

fn load_config(cli: &Cli) -> Result<crate::config::schema::AppConfig> { 
    let config = crate::config::parse::load_config(Path::new(&cli.config))?; 
    Ok(config) 
}