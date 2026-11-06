//! Layout of the data directory.

use std::path::{Path, PathBuf};

pub fn database(data_dir: &Path) -> PathBuf {
    data_dir.join("hub.db")
}

pub fn secret_key(data_dir: &Path) -> PathBuf {
    data_dir.join("keys").join("secret.key")
}

pub fn generated_certificate(data_dir: &Path) -> PathBuf {
    data_dir.join("tls").join("certificate.pem")
}

pub fn generated_private_key(data_dir: &Path) -> PathBuf {
    data_dir.join("tls").join("private-key.pem")
}
