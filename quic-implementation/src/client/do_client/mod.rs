use quinn::Endpoint;
use rustls::client::ServerCertVerified;
use rustls::{Certificate, ServerName};
use std::time::SystemTime;
use std::{
    error::Error,
    fs,
    io::{self, Write},
    net::ToSocketAddrs,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use tracing::{error, info};
use url::Url;

use super::super::commons;
use super::env_parser::Config;

#[tokio::main]
pub async fn do_client(config: &Config) -> Result<(), Box<dyn Error>> {
    let urls = get_urls(config);
    let url = urls.get(0).unwrap();
    let remote = (url.host_str().unwrap(), url.port().unwrap_or(4433))
        .to_socket_addrs()?
        .next()
        .unwrap();

    let tls_config_builder = rustls::ClientConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_protocol_versions(&[&rustls::version::TLS13])?;
    let mut tls_config = tls_config_builder
        .with_custom_certificate_verifier(Arc::new(MyCustomVerifier))
        .with_no_client_auth();
    tls_config.alpn_protocols = commons::ALPN_QUIC_HTTP.iter().map(|&x| x.into()).collect();
    let client_config = quinn::ClientConfig::new(Arc::new(tls_config));

    let mut endpoint = quinn::Endpoint::client("[::]:0".parse().unwrap())?;
    endpoint.set_default_client_config(client_config);

    let request = format!("GET {}\r\n", url.path());
    let start = Instant::now();
    let host = url.host_str().unwrap();
    eprintln!("connecting to {} at {}", host, remote);

    let new_conn = endpoint.connect(remote, host)?.await?;
    eprintln!("connected at {:?}", start.elapsed());

    let quinn::NewConnection {
        connection: conn, ..
    } = new_conn;
    let (mut send, recv) = conn.open_bi().await?;

    send.write_all(request.as_bytes()).await?;
    send.finish().await?;
    let response_start = Instant::now();
    eprintln!("request sent at {:?}", response_start - start);
    let resp = recv.read_to_end(usize::max_value()).await?;
    let duration = response_start.elapsed();
    eprintln!(
        "response received in {:?} - {} KiB/s",
        duration,
        resp.len() as f32 / (duration_secs(&duration) * 1024.0)
    );
    io::stdout().write_all(&resp).unwrap();
    io::stdout().flush().unwrap();
    conn.close(0u32.into(), b"done");

    // Give the server a fair chance to receive the close packet
    endpoint.wait_idle().await;

    Ok(())
}

fn get_urls(config: &Config) -> Vec<Url> {
    config
        .requests
        .iter()
        .map(|url| Url::parse(url).unwrap())
        .collect()
}

fn duration_secs(x: &Duration) -> f32 {
    x.as_secs() as f32 + x.subsec_nanos() as f32 * 1e-9
}

struct MyCustomVerifier;
impl rustls::client::ServerCertVerifier for MyCustomVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
}
