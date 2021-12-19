use quic_implementation::certs_configuration;
use quic_implementation::env_parser::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new();
    println!("Config is: {:#?}", config);

    let res = certs_configuration::parse_certificates(&config)?;
    println!("{:#?}", res);

    Ok(())
}
