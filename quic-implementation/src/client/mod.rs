use std::sync::Arc;

use futures::future;
use h3_quinn::quinn;
use tokio::{self, io::AsyncWriteExt};
use tracing::info;

mod certs_configuration;
mod env_parser;

pub async fn run_client() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        // .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .with_writer(std::io::stderr)
        .init();

    let config = env_parser::Config::new();
    let uri = config.requests.get(0).unwrap();
    let dest = uri.parse::<http::Uri>()?;

    if dest.scheme() != Some(&http::uri::Scheme::HTTPS) {
        Err("destination scheme must be 'https'")?;
    }

    let auth = dest
        .authority()
        .ok_or("destination must have a host")?
        .clone();

    let port = auth.port_u16().unwrap_or(443);

    let addr = tokio::net::lookup_host((auth.host(), port))
        .await?
        .next()
        .ok_or("dns found no addresses")?;

    info!("DNS Lookup for {:?}: {:?}", dest, addr);

    let client_crypto = certs_configuration::get_client_crypto()?;
    let client_config = quinn::ClientConfig::new(Arc::new(client_crypto));

    let mut client_endpoint = h3_quinn::quinn::Endpoint::client("[::]:0".parse().unwrap())?;
    client_endpoint.set_default_client_config(client_config);
    let quinn_conn = h3_quinn::Connection::new(client_endpoint.connect(addr, auth.host())?.await?);

    info!("QUIC connected ...");

    // generic h3
    let (mut driver, mut send_request) = h3::client::new(quinn_conn).await?;

    let drive = async move {
        future::poll_fn(|cx| driver.poll_close(cx)).await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    };

    let request = async move {
        info!("Sending request ...");

        let req = http::Request::builder().uri(dest).body(())?;

        let mut stream = send_request.send_request(req).await?;
        stream.finish().await?;

        info!("Receiving response ...");
        let resp = stream.recv_response().await?;

        info!("Response: {:?} {}", resp.version(), resp.status());
        info!("Headers: {:#?}", resp.headers());

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

    client_endpoint.wait_idle().await;

    Ok(())
}
