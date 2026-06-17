use clap::Parser; 
use clap::Subcommand; 


#[derive(Parser, Debug)] 
#[command(name = "peregrine")] 
#[command(about = "a Pingora gateway")] 
pub struct Cli { 
    #[arg(short, long, default_value = "peregrine.yaml")] 
    pub config: String, 

    #[command(subcommand)] 
    pub command: Option<Command>, 
}

#[derive(Subcommand, Debug)] 
pub enum Command { 
    Validate, 
    Init, 
}
