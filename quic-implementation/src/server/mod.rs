pub mod certs_configuration;
pub mod env_parser;
pub mod server_setup;

use env_parser::Config;

pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new();
    println!("Config is: {:#?}", config);

    let server_crypto = certs_configuration::get_server_crypto(&config)?;
    server_setup::start_server(&config, server_crypto)?;

    Ok(())
}
