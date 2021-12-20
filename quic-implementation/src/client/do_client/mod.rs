use std::time::SystemTime;
use std::{
    error::Error,
    net::ToSocketAddrs,
    sync::Arc,
    time::{Duration, Instant},
};

use futures::future;
use h3_quinn::quinn;
use http::Uri;
use rustls::client::ServerCertVerified;
use rustls::{Certificate, ServerName};
use tokio::{self, io::AsyncWriteExt};

use super::super::commons;
use super::env_parser::Config;

#[tokio::main]
pub async fn do_client(config: &Config) -> Result<(), Box<dyn Error>> {
    let uris = get_uris(config);
    let dest = uris.get(0).unwrap();
    if dest.scheme() != Some(&http::uri::Scheme::HTTPS) {
        Err("destination scheme must be 'https'")?;
    }

    let auth = dest
        .authority()
        .ok_or("destination must have a host")?
        .clone();

    let port = auth.port_u16().unwrap_or(443);

    let addr = match tokio::net::lookup_host((auth.host(), port)).await?.next() {
        Some(addr) => addr,
        None => (auth.host(), port).to_socket_addrs()?.next().unwrap(),
    };

    let tls_config_builder = rustls::ClientConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_protocol_versions(&[&rustls::version::TLS13])?;
    let mut tls_config = tls_config_builder
        .with_custom_certificate_verifier(Arc::new(MyCustomVerifier))
        .with_no_client_auth();
    tls_config.enable_early_data = true;
    tls_config.alpn_protocols = commons::ALPN_QUIC_HTTP.iter().map(|&x| x.into()).collect();
    let client_config = quinn::ClientConfig::new(Arc::new(tls_config));

    let mut endpoint = h3_quinn::quinn::Endpoint::client("[::]:0".parse().unwrap())?;
    endpoint.set_default_client_config(client_config);

    let start = Instant::now();
    let host = auth.host();
    eprintln!("QUIC connecting to {} at {}", host, dest);

    let quinn_conn = h3_quinn::Connection::new(endpoint.connect(addr, host)?.await?);
    eprintln!("QUIC connected at {:?}", start.elapsed());

    let (mut driver, mut send_request) = h3::client::new(quinn_conn).await?;
    let drive = async move {
        future::poll_fn(|cx| driver.poll_close(cx)).await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    };

    let request = async move {
        eprintln!("Sending request ...");

        let req = http::Request::builder().uri(dest).body(())?;

        let mut stream = send_request.send_request(req).await?;
        stream.finish().await?;

        eprintln!("Receiving response ...");
        let resp = stream.recv_response().await?;

        eprintln!("Response: {:?} {}", resp.version(), resp.status());
        eprintln!("Headers: {:#?}", resp.headers());

        while let Some(chunk) = stream.recv_data().await? {
            let mut out = tokio::io::stdout();
            out.write_all(&chunk).await.expect("write_all");
            out.flush().await.expect("flush");
        }
        Ok::<_, Box<dyn std::error::Error>>(())
    };

    let (req_res, drive_res) = tokio::join!(request, drive);
    req_res?;
    drive_res?;

    endpoint.wait_idle().await;

    // let quinn::NewConnection {
    //     connection: conn, ..
    // } = new_conn;
    // let (mut send, recv) = conn.open_bi().await?;

    // send.write_all(request.as_bytes()).await?;
    // send.finish().await?;
    // let response_start = Instant::now();
    // eprintln!("request sent at {:?}", response_start - start);
    // let resp = recv.read_to_end(usize::max_value()).await?;
    // let duration = response_start.elapsed();
    // eprintln!(
    //     "response received in {:?} - {} KiB/s",
    //     duration,
    //     resp.len() as f32 / (duration_secs(&duration) * 1024.0)
    // );
    // io::stdout().write_all(&resp).unwrap();
    // io::stdout().flush().unwrap();
    // conn.close(0u32.into(), b"done");

    // // Give the server a fair chance to receive the close packet
    // endpoint.wait_idle().await;

    Ok(())
}

fn get_uris(config: &Config) -> Vec<Uri> {
    config
        .requests
        .iter()
        .map(|url| url.parse::<http::Uri>().unwrap())
        .collect()
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
