use std::error::Error;
use std::net::{IpAddr, SocketAddr};
use std::path::Path;
use std::sync::Arc;
use std::{
    ascii, fs, io,
    path::{self, PathBuf},
    str,
};

use anyhow::{anyhow, bail, Context, Result};
use futures_util::StreamExt;
use quinn::ServerConfig;
use rustls::ServerConfig as CryptoConfig;

use super::env_parser::Config;

#[tokio::main]
pub async fn do_server(config: &Config, server_crypto: CryptoConfig) -> Result<(), Box<dyn Error>> {
    let server_config = get_server_config(server_crypto);
    let www = get_www(config);
    let listen_address = get_listen_address(config);

    let (endpoint, mut incoming) = quinn::Endpoint::server(server_config, listen_address)?;
    println!("listening on {}", endpoint.local_addr()?);

    while let Some(conn) = incoming.next().await {
        println!("connection incoming");
        tokio::spawn(
            handle_connection(www.clone(), conn),
        );
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

async fn handle_connection(root: Arc<Path>, conn: quinn::Connecting) -> () {
    let quinn::NewConnection {
        connection,
        mut bi_streams,
        ..
    } = conn.await.unwrap();
    // let span = info_span!(
    //     "connection",
    //     remote = %connection.remote_address(),
    //     protocol = %connection
    //         .handshake_data()
    //         .unwrap()
    //         .downcast::<quinn::crypto::rustls::HandshakeData>().unwrap()
    //         .protocol
    //         .map_or_else(|| "<none>".into(), |x| String::from_utf8_lossy(&x).into_owned())
    // );
    async {
        while let Some(stream) = bi_streams.next().await {
            let stream = match stream {
                Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                    return Ok(());
                }
                Err(e) => {
                    return Err(e);
                }
                Ok(s) => s,
            };
            tokio::spawn(handle_request(root.clone(), stream));
        }
        Ok(())
    }
    .await
    .unwrap();
}

async fn handle_request(
    root: Arc<Path>,
    (mut send, recv): (quinn::SendStream, quinn::RecvStream),
) -> Result<()> {
    let req = recv
        .read_to_end(64 * 1024)
        .await
        .map_err(|e| anyhow!("failed reading request: {}", e))?;
    let mut escaped = String::new();
    for &x in &req[..] {
        let part = ascii::escape_default(x).collect::<Vec<_>>();
        escaped.push_str(str::from_utf8(&part).unwrap());
    }
    // Execute the request
    let resp = process_get(&root, &req).unwrap_or_else(|e| {
        format!("failed to process request: {}\n", e).into_bytes()
    });
    // Write the response
    send.write_all(&resp)
        .await
        .map_err(|e| anyhow!("failed to send response: {}", e))?;
    // Gracefully terminate the stream
    send.finish()
        .await
        .map_err(|e| anyhow!("failed to shutdown stream: {}", e))?;
    Ok(())
}

fn process_get(root: &Path, x: &[u8]) -> Result<Vec<u8>> {
    if x.len() < 4 || &x[0..4] != b"GET " {
        bail!("missing GET");
    }
    if x[4..].len() < 2 || &x[x.len() - 2..] != b"\r\n" {
        bail!("missing \\r\\n");
    }
    let x = &x[4..x.len() - 2];
    let end = x.iter().position(|&c| c == b' ').unwrap_or_else(|| x.len());
    let path = str::from_utf8(&x[..end]).context("path is malformed UTF-8")?;
    let path = Path::new(&path);
    let mut real_path = PathBuf::from(root);
    let mut components = path.components();
    match components.next() {
        Some(path::Component::RootDir) => {}
        _ => {
            bail!("path must be absolute");
        }
    }
    for c in components {
        match c {
            path::Component::Normal(x) => {
                real_path.push(x);
            }
            x => {
                bail!("illegal component in path: {:?}", x);
            }
        }
    }
    let data = fs::read(&real_path).context("failed reading file")?;
    Ok(data)
}
