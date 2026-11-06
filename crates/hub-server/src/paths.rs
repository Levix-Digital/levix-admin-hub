//! Layout of the data directory.

use std::path::{Path, PathBuf};

use anyhow::Context;

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

/// Creates the data directory readable by its owner only; the database and the keys inside
/// must not be readable by other users of the host.
pub fn create_data_dir(data_dir: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(data_dir)
        .with_context(|| format!("creating the data directory {}", data_dir.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(data_dir, std::fs::Permissions::from_mode(0o700))
            .with_context(|| format!("restricting {} to its owner", data_dir.display()))?;
    }
    Ok(())
}
