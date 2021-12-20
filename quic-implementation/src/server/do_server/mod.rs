use std::error::Error;
use std::net::{IpAddr, SocketAddr};
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use bytes::Bytes;
use futures_util::StreamExt;
use h3::quic::BidiStream;
use h3::server::RequestStream;
use h3_quinn::quinn::{self, ServerConfig};
use rustls::ServerConfig as CryptoConfig;

use super::env_parser::Config;

#[tokio::main]
pub async fn do_server(config: &Config, server_crypto: CryptoConfig) -> Result<(), Box<dyn Error>> {
    let server_config = get_server_config(server_crypto);
    let www = get_www(config);
    let listen_address = get_listen_address(config);

    let (endpoint, mut incoming) = h3_quinn::quinn::Endpoint::server(server_config, listen_address)?;
    println!("listening on {}", endpoint.local_addr()?);

    while let Some(conn) = incoming.next().await {
        println!("connection incoming");
        tokio::spawn(handle_connection(www.clone(), conn));
    }

    Ok(())
}

fn get_server_config(server_crypto: CryptoConfig) -> ServerConfig {
    let mut server_config = quinn::ServerConfig::with_crypto(Arc::new(server_crypto));
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

async fn handle_connection(_www: Arc<Path>, conn: quinn::Connecting) -> () {
    match conn.await {
        Ok(conn) => {
            let mut h3_conn = h3::server::Connection::new(h3_quinn::Connection::new(conn))
                .await
                .unwrap();

            while let Some((_req, stream)) = h3_conn.accept().await.unwrap() {
                tokio::spawn(handle_request(stream));
            }
        }
        Err(err) => {
            println!("connecting client failed with error: {:?}", err);
        }
    }
}

async fn handle_request<T>(
    mut stream: RequestStream<T>,
) -> Result<(), Box<dyn std::error::Error + Send>>
where
    T: BidiStream<Bytes>,
{
    let resp = http::Response::builder()
        .status(http::StatusCode::NOT_FOUND)
        .body(())
        .unwrap();

    match stream.send_response(resp).await {
        Ok(_) => {
            println!("Response to connection successful");
        }
        Err(err) => {
            println!("Unable to send response to connection peer: {:?}", err);
        }
    }

    Ok(stream.finish().await?)
}
