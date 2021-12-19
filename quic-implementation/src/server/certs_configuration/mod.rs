use std::error::Error;
use std::sync::Arc;
use std::{fs, path::Path};

use rustls::{Certificate, PrivateKey, ServerConfig};

use super::super::commons;
use super::env_parser::Config;

pub fn get_server_crypto(config: &Config) -> Result<ServerConfig, Box<dyn Error>> {
    let (certs, key) = parse_certificates(config)?;

    let mut server_crypto = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;

    server_crypto.alpn_protocols = commons::ALPN_QUIC_HTTP.iter().map(|&x| x.into()).collect();
    server_crypto.key_log = Arc::new(rustls::KeyLogFile::new());

    Ok(server_crypto)
}

fn parse_certificates(config: &Config) -> Result<(Vec<Certificate>, PrivateKey), Box<dyn Error>> {
    let certs_dir = Path::new(&config.certs);
    let cert_path = certs_dir.join("cert.pem");
    let key_path = certs_dir.join("key.pem");

    let (cert_chain, key) = fs::read(&cert_path).and_then(|x| Ok((x, fs::read(&key_path)?)))?;
    parse_pem(cert_chain, key)
}

fn parse_pem(
    cert: Vec<u8>,
    private_key: Vec<u8>,
) -> Result<(Vec<Certificate>, PrivateKey), Box<dyn Error>> {
    let parsed_key = parse_pem_key(private_key)?;
    let parsed_certs = parse_pem_cert(cert)?;

    Ok((parsed_certs, parsed_key))
}

fn parse_pem_cert(cert: Vec<u8>) -> Result<Vec<Certificate>, Box<dyn Error>> {
    let v: Vec<Certificate> = rustls_pemfile::certs(&mut &*cert)?
        .into_iter()
        .map(rustls::Certificate)
        .collect();

    Ok(v)
}

fn parse_pem_key(key: Vec<u8>) -> Result<PrivateKey, Box<dyn Error>> {
    let pkcs8: Vec<Vec<u8>> = rustls_pemfile::pkcs8_private_keys(&mut &*key)?;
    let key = match pkcs8.into_iter().next() {
        Some(x) => PrivateKey(x),
        None => {
            let rsa = rustls_pemfile::rsa_private_keys(&mut &*key)?;
            match rsa.into_iter().next() {
                Some(x) => PrivateKey(x),
                None => panic!("no private key found"),
            }
        }
    };
    Ok(key)
}
