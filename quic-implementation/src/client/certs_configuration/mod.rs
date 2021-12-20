use rustls::client::ServerCertVerified;
use rustls::ClientConfig;
use rustls::{Certificate, ServerName};
use std::error::Error;
use std::sync::Arc;
use std::time::SystemTime;

use super::super::commons;

pub fn get_client_crypto() -> Result<ClientConfig, Box<dyn Error>> {
    let tls_config_builder = ClientConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_protocol_versions(&[&rustls::version::TLS13])?;
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
