use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use std::{
    ascii, fs, io,
    net::SocketAddr,
    path::{self, PathBuf},
    str,
};

use bytes::Bytes;
use futures::StreamExt;
use h3::{quic::BidiStream, server::RequestStream};
use rustls::{Certificate, PrivateKey};

mod certs_configuration;
mod env_parser;
mod setup_logs;

pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let config = env_parser::Config::new();
    println!("{:#?}", config);

    setup_logs::setup_logs(&config);

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
        println!("New connection being attempted");
        let www = config.www.clone();

        tokio::spawn(async move {
            match new_conn.await {
                Ok(conn) => {
                    println!("New connection now established");

                    let mut h3_conn = h3::server::Connection::new(h3_quinn::Connection::new(conn))
                        .await
                        .unwrap();

                    while let Some((req, stream)) = h3_conn.accept().await.unwrap() {
                        println!("connection requested: {:#?}", req);

                        tokio::spawn(handle_request(www.clone(), req, stream));
                    }
                }
                Err(err) => {
                    println!("connecting client failed with error: {:?}", err);
                }
            }
        });
    }

    Ok(())
}

async fn handle_request<T>(
    www: String,
    req: http::Request<()>,
    mut stream: RequestStream<T>,
) -> Result<(), Box<dyn std::error::Error + Send>>
where
    T: BidiStream<Bytes>,
{
    let www_path = Path::new(&www);
    let uri = http::Uri::try_from(req.uri()).unwrap();
    let requested_file = uri.path();
    let (_, requested_file) = requested_file.split_at(1);
    let file_path = www_path.join(requested_file);
    println!(
        "PORCAMERDOSAAAAAAAAAAAAAAAAAAAAAAAAAAA {}",
        file_path.to_str().unwrap()
    );
    let path_total = req.uri().clone().into_parts().path_and_query.unwrap();
    let path_total = path_total.path();

    if !file_path.exists() {
        println!("File not found: {:?}", file_path);

        let response = http::Response::builder()
            .status(http::StatusCode::NOT_FOUND)
            .body(())
            .unwrap();

        match stream.send_response(response).await {
            Ok(_) => {
                println!("Response to connection successful");
            }
            Err(err) => {
                println!("Unable to send response to connection peer: {:?}", err);
            }
        }
    } else {
        let response = http::Response::builder()
            .status(http::StatusCode::OK)
            .body(())
            .unwrap();

        let file = process_get(www_path, path_total).unwrap();

        match stream.send_response(response).await {
            Ok(_) => {
                println!("Response to connection successful");
            }
            Err(err) => {
                println!("Unable to send response to connection peer: {:?}", err);
            }
        }

        match stream.send_data(Bytes::from(file)).await {
            Ok(_) => {
                println!("Response to connection successful");
            }
            Err(err) => {
                println!("Unable to send response to connection peer: {:?}", err);
            }
        }
    };

    Ok(stream.finish().await?)
}

fn process_get(root: &Path, path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let path = Path::new(&path);
    let mut real_path = PathBuf::from(root);
    let mut components = path.components();
    match components.next() {
        Some(path::Component::RootDir) => {}
        _ => {
            println!("path must be absolute");
        }
    }
    for c in components {
        match c {
            path::Component::Normal(x) => {
                real_path.push(x);
            }
            x => {
                println!("illegal component in path: {:?}", x);
            }
        }
    }
    let data = fs::read(&real_path)?;
    Ok(data)
}
