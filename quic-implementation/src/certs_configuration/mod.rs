use std::path;
use anyhow::result;
use rustls::{Certificate, CertificateChain, PrivateKey, PrivateKey};

use quic_implementation::env_parser::Config;

fn parse_pem(cert: Vec<u8>, private_key: Vec<u8>) -> anyhow::Result<(Certificate, PrivateKey)> {
    // Parse to certificate chain whereafter taking the first certifcater in this chain.
    let cert = CertificateChain::from_pem(&cert)?
        .iter()
        .next()
        .unwrap()
        .clone();
    let key = PrivateKey::from_pem(&private_key)?;

    Ok((Certificate::from(cert), key))
}

fn parse_certificates(config: &Config) -> anyhow::Result<(Certificate, PrivateKey)> {
    let certs_dir = path::Path(&config.certs);
    let cert_path = certs_dir.join("cert.pem").to_str();
    let key_path = certs_dir.join("key.pem").to_str();

    let (cert, key) = fs::read(&cert_path).and_then(|x| Ok((x, fs::read(&key_path)?)))?;
    parse_pem(cert, key)
}
