use rustls::client::ServerCertVerified;
use rustls::{Certificate, ServerName};
use rustls::{ClientConfig, cipher_suite::TLS13_CHACHA20_POLY1305_SHA256};
use std::error::Error;
use std::sync::Arc;
use std::time::SystemTime;

use super::super::commons;
use super::env_parser::Config;

pub fn get_client_crypto(testcase: &String) -> Result<ClientConfig, Box<dyn Error>> {
    let tls_config_builder = if testcase == "chacha20"  {
        let cipher_suites = [TLS13_CHACHA20_POLY1305_SHA256];
        ClientConfig::builder()
            .with_cipher_suites(&cipher_suites)
            .with_safe_default_kx_groups()
            .with_protocol_versions(&[&rustls::version::TLS13])?
    } else {
        ClientConfig::builder()
            .with_safe_default_cipher_suites()
            .with_safe_default_kx_groups()
            .with_protocol_versions(&[&rustls::version::TLS13])?
    };
    let mut tls_config = tls_config_builder
        .with_custom_certificate_verifier(Arc::new(YesVerifier))
        .with_no_client_auth();
    tls_config.enable_early_data = true;
    tls_config.alpn_protocols = vec![commons::ALPN.into()];
    tls_config.key_log = Arc::new(rustls::KeyLogFile::new());

    Ok(tls_config)
}

struct YesVerifier;

impl rustls::client::ServerCertVerifier for YesVerifier {
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
