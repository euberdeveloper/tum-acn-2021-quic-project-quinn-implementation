use std::error::Error;
use std::net::{IpAddr, SocketAddr};
use std::path::Path;
use std::sync::Arc;

use futures_util::{StreamExt};
use quinn::ServerConfig;
use rustls::ServerConfig as CryptoConfig;

use super::env_parser::Config;

#[tokio::main]
pub async fn start_server(config: &Config, server_crypto: CryptoConfig) -> Result<(), Box<dyn Error>> {
    let server_config = get_server_config(server_crypto);
    let _www = get_www(config);
    let listen_address = get_listen_address(config);

    let (endpoint, mut incoming) = quinn::Endpoint::server(server_config, listen_address)?;
    println!("listening on {}", endpoint.local_addr()?);

    while let Some(_conn) = incoming.next().await {
        println!("porcu");
    }

    Ok(())
}

fn get_server_config(server_crypto: CryptoConfig) -> ServerConfig {
    let mut server_config = ServerConfig::with_crypto(Arc::new(server_crypto));
    Arc::get_mut(&mut server_config.transport)
        .unwrap()
        .max_concurrent_uni_streams(0_u8.into());

    server_config
}

fn get_www(config: &Config) -> Arc<Path> {
    let www = Path::new(&config.www);

    if !www.exists() {
        panic!("www directory does not exist");
    }

    Arc::from(www)
}

fn get_listen_address(config: &Config) -> SocketAddr {
    let address: IpAddr = config.ip.parse().expect("invalid ip address");
    let port: u16 = config.port.parse().expect("invalid port");
    let listen = SocketAddr::from((address, port));

    listen
}
