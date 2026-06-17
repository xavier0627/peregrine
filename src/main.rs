mod cli; 
mod config; 
mod handler; 
mod proxy; 
mod server; 
mod util;
mod filter;


use clap::Parser; 
use cli::args::Cli; 

fn main() -> anyhow::Result<()> { 
    let cli = Cli::parse(); 
    cli::dispatch(cli) 
}