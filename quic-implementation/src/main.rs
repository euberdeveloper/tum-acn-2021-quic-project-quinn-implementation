use quic_implementation::certs_configuration;
use quic_implementation::env_parser::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new();
    println!("Config is: {:#?}", config);

    let server_config = certs_configuration::get_certificate_config(&config)?;
    println!("haho");

    Ok(())
}
