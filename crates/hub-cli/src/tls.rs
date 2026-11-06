//! How the CLI trusts the Hub: the system's certificate authorities (plus an optional CA
//! file), or the SHA-256 fingerprint of the Hub certificate that `init` printed.

use std::path::Path;
use std::sync::Arc;

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::{verify_tls12_signature, verify_tls13_signature, CryptoProvider};
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, RootCertStore, SignatureScheme};
use sha2::{Digest, Sha256};

use crate::error::{CliError, CliResult};

/// Normalizes a fingerprint written as `AB:CD:…`, `abcd…` or `sha256:abcd…` to 64 hex digits.
pub fn normalize_fingerprint(text: &str) -> CliResult<String> {
    let hex: String = text
        .trim()
        .trim_start_matches("sha256:")
        .trim_start_matches("SHA256:")
        .chars()
        .filter(|c| *c != ':')
        .collect::<String>()
        .to_ascii_lowercase();
    if hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        Ok(hex)
    } else {
        Err(CliError::Usage(
            "the fingerprint must be the 64 hex digits of a SHA-256 digest".to_owned(),
        ))
    }
}

fn provider() -> Arc<CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}

/// The TLS configuration for one Hub.
pub fn client_config(fingerprint: Option<&str>, ca_file: Option<&Path>) -> CliResult<ClientConfig> {
    let builder = ClientConfig::builder_with_provider(provider())
        .with_safe_default_protocol_versions()
        .map_err(|e| CliError::Local(e.to_string()))?;
    let config = match fingerprint {
        Some(fingerprint) => builder
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(Pinned {
                fingerprint: normalize_fingerprint(fingerprint)?,
                provider: provider(),
            }))
            .with_no_client_auth(),
        None => {
            let mut roots = RootCertStore::empty();
            for certificate in rustls_native_certs::load_native_certs().certs {
                let _ignored = roots.add(certificate);
            }
            if let Some(path) = ca_file {
                let certificates = CertificateDer::pem_file_iter(path)
                    .and_then(|iter| iter.collect::<Result<Vec<_>, _>>())
                    .map_err(|e| CliError::Usage(format!("{}: {e}", path.display())))?;
                for certificate in certificates {
                    roots
                        .add(certificate)
                        .map_err(|e| CliError::Usage(format!("{}: {e}", path.display())))?;
                }
            }
            builder.with_root_certificates(roots).with_no_client_auth()
        }
    };
    Ok(config)
}

/// Accepts exactly the certificate whose SHA-256 digest was pinned; handshake signatures
/// are still verified, so only the holder of that certificate's key can answer.
#[derive(Debug)]
struct Pinned {
    fingerprint: String,
    provider: Arc<CryptoProvider>,
}

impl ServerCertVerifier for Pinned {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        let digest: String = Sha256::digest(end_entity.as_ref())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        if digest == self.fingerprint {
            Ok(ServerCertVerified::assertion())
        } else {
            Err(rustls::Error::General(format!(
                "the Hub certificate fingerprint is {digest}, not the expected one; \
                 check it on the Hub host"
            )))
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls12_signature(
            message,
            cert,
            dss,
            &self.provider.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls13_signature(
            message,
            cert,
            dss,
            &self.provider.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.provider
            .signature_verification_algorithms
            .supported_schemes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprints_normalize() {
        let hex = "ab".repeat(32);
        let colons = vec!["AB"; 32].join(":");
        assert_eq!(normalize_fingerprint(&colons).ok(), Some(hex.clone()));
        assert_eq!(
            normalize_fingerprint(&format!("sha256:{hex}")).ok(),
            Some(hex)
        );
        assert!(normalize_fingerprint("abc").is_err());
    }
}
