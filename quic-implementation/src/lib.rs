pub mod certs_configuration;
pub mod env_parser;
pub mod server;

use env_parser::Config;

pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new();
    println!("Config is: {:#?}", config);

    let server_crypto = certs_configuration::get_server_crypto(&config)?;
    server::start_server(&config, server_crypto)?;

    Ok(())
}

pub async fn run_client() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new();
    println!("Config is: {:#?}", config);

    let server_crypto = certs_configuration::get_server_crypto(&config)?;
    server::start_server(&config, server_crypto)?;

    Ok(())
}
