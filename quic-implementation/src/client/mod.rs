pub mod env_parser;
pub mod do_client;

use env_parser::Config;

pub async fn run_client() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new();
    println!("Config is: {:#?}", config);

    do_client::do_client(&config)?;
    
    Ok(())
}
