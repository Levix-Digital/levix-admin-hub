//! TLS for every endpoint (TLS 1.2 or later through rustls): the customer's certificate, or a
//! self-signed one generated into the data directory, whose SHA-256 fingerprint is printed so
//! administrators and the CLI can verify it.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use axum_server::tls_rustls::RustlsConfig;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use sha2::{Digest, Sha256};

use crate::cli::TlsArgs;
use crate::paths;

/// Certificate and key files the server will use.
#[derive(Debug, Clone)]
pub struct TlsFiles {
    pub certificate: PathBuf,
    pub private_key: PathBuf,
    pub generated: bool,
}

/// Picks the customer's files, or generates a self-signed certificate once and reuses it.
pub fn prepare(data_dir: &Path, args: &TlsArgs) -> anyhow::Result<TlsFiles> {
    if let (Some(certificate), Some(private_key)) = (&args.tls_cert, &args.tls_key) {
        return Ok(TlsFiles {
            certificate: certificate.clone(),
            private_key: private_key.clone(),
            generated: false,
        });
    }
    let certificate = paths::generated_certificate(data_dir);
    let private_key = paths::generated_private_key(data_dir);
    if !certificate.exists() {
        generate(&certificate, &private_key, &args.hostnames)?;
    }
    Ok(TlsFiles {
        certificate,
        private_key,
        generated: true,
    })
}

fn generate(certificate: &Path, private_key: &Path, hostnames: &[String]) -> anyhow::Result<()> {
    if hostnames.is_empty() {
        bail!("at least one host name is needed to generate a certificate");
    }
    let generated = rcgen::generate_simple_self_signed(hostnames.to_vec())
        .context("generating the self-signed certificate")?;
    if let Some(parent) = certificate.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    write_private(private_key, generated.key_pair.serialize_pem().as_bytes())?;
    fs::write(certificate, generated.cert.pem())
        .with_context(|| format!("writing {}", certificate.display()))?;
    Ok(())
}

fn write_private(path: &Path, contents: &[u8]) -> anyhow::Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(path)
        .with_context(|| format!("writing {}", path.display()))?;
    file.write_all(contents)
        .with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

/// SHA-256 fingerprint of the first certificate of the chain, as `AB:CD:...`.
pub fn fingerprint(certificate: &Path) -> anyhow::Result<String> {
    let first = CertificateDer::pem_file_iter(certificate)
        .with_context(|| format!("reading {}", certificate.display()))?
        .next()
        .with_context(|| format!("no certificate in {}", certificate.display()))?
        .with_context(|| format!("parsing {}", certificate.display()))?;
    Ok(format_fingerprint(&Sha256::digest(first.as_ref())))
}

pub fn format_fingerprint(digest: &[u8]) -> String {
    digest
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(":")
}

/// rustls configuration for the server, TLS 1.2 and 1.3 only.
pub async fn server_config(files: &TlsFiles) -> anyhow::Result<RustlsConfig> {
    let chain = CertificateDer::pem_file_iter(&files.certificate)
        .with_context(|| format!("reading {}", files.certificate.display()))?
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("parsing {}", files.certificate.display()))?;
    let key = PrivateKeyDer::from_pem_file(&files.private_key)
        .with_context(|| format!("reading {}", files.private_key.display()))?;
    let provider = std::sync::Arc::new(rustls::crypto::ring::default_provider());
    let mut config = rustls::ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])
        .context("selecting TLS versions")?
        .with_no_client_auth()
        .with_single_cert(chain, key)
        .context("loading the certificate and its key")?;
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    Ok(RustlsConfig::from_config(std::sync::Arc::new(config)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args() -> TlsArgs {
        TlsArgs {
            tls_cert: None,
            tls_key: None,
            hostnames: vec!["hub.example".to_owned()],
        }
    }

    #[test]
    fn generates_once_and_keeps_the_fingerprint() {
        let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
        let first = prepare(dir.path(), &args()).unwrap_or_else(|e| panic!("{e}"));
        let fingerprint_1 = fingerprint(&first.certificate).unwrap_or_else(|e| panic!("{e}"));
        let second = prepare(dir.path(), &args()).unwrap_or_else(|e| panic!("{e}"));
        let fingerprint_2 = fingerprint(&second.certificate).unwrap_or_else(|e| panic!("{e}"));
        assert!(first.generated);
        assert_eq!(fingerprint_1, fingerprint_2);
        assert_eq!(fingerprint_1.len(), 32 * 3 - 1);
    }

    #[cfg(unix)]
    #[test]
    fn generated_key_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
        let files = prepare(dir.path(), &args()).unwrap_or_else(|e| panic!("{e}"));
        let mode = fs::metadata(&files.private_key)
            .unwrap_or_else(|e| panic!("{e}"))
            .permissions()
            .mode();
        assert_eq!(mode & 0o077, 0);
    }

    #[tokio::test]
    async fn builds_a_server_configuration() {
        let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
        let files = prepare(dir.path(), &args()).unwrap_or_else(|e| panic!("{e}"));
        assert!(server_config(&files).await.is_ok());
    }
}
