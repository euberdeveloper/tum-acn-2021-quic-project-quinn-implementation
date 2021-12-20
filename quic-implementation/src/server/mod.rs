use std::sync::Arc;

use bytes::Bytes;
use futures::StreamExt;
use h3::{quic::BidiStream, server::RequestStream};
use rustls::{Certificate, PrivateKey};
use tracing::{debug, error, info, trace, trace_span, warn};

mod env_parser;
mod certs_configuration;

pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .with_writer(std::io::stderr)
        .init();

    let config = env_parser::Config::new();
    trace!("{:#?}", config);

    
    let crypto = certs_configuration::get_server_crypto(&config)?;
    let server_config = h3_quinn::quinn::ServerConfig::with_crypto(Arc::new(crypto));

    let port = config.port;
    let addr = format!("[::]:{:}", port).parse()?;
    let (endpoint, mut incoming) = h3_quinn::quinn::Endpoint::server(server_config, addr)?;

    println!(
        "Listening on port {:?}",
        endpoint.local_addr().unwrap().port()
    );

    while let Some(new_conn) = incoming.next().await {
        trace_span!("New connection being attempted");

        tokio::spawn(async move {
            match new_conn.await {
                Ok(conn) => {
                    debug!("New connection now established");

                    let mut h3_conn = h3::server::Connection::new(h3_quinn::Connection::new(conn))
                        .await
                        .unwrap();

                    while let Some((req, stream)) = h3_conn.accept().await.unwrap() {
                        debug!("connection requested: {:#?}", req);

                        tokio::spawn(handle_request(stream));
                    }
                }
                Err(err) => {
                    warn!("connecting client failed with error: {:?}", err);
                }
            }
        });
    }

    Ok(())
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
            debug!("Response to connection successful");
        }
        Err(err) => {
            error!("Unable to send response to connection peer: {:?}", err);
        }
    }

    Ok(stream.finish().await?)
}

pub fn build_certs() -> (Certificate, PrivateKey) {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let key = PrivateKey(cert.serialize_private_key_der());
    let cert = Certificate(cert.serialize_der().unwrap());
    (cert, key)
}
