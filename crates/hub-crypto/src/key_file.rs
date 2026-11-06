//! The secret key kept in a file outside the database (security architecture, Secrets and
//! keys). It is created once by `levix-hub init` with owner-only permissions and must exist
//! for the Hub to start.

use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};

use rand::rngs::OsRng;
use rand::RngCore;
use thiserror::Error;

/// Length of the secret key in bytes (256 bits).
pub const SECRET_KEY_LEN: usize = 32;

/// Failures of the key file, with the path for the operator.
#[derive(Debug, Error)]
pub enum KeyFileError {
    #[error("the secret key file {0} already exists; it is never overwritten")]
    AlreadyExists(PathBuf),
    #[error("the secret key file {0} is missing; run `levix-hub init` on the host")]
    Missing(PathBuf),
    #[error("the secret key file {path} must be {SECRET_KEY_LEN} bytes, found {found}")]
    WrongLength { path: PathBuf, found: usize },
    #[error("the secret key file {path} is readable by other users (mode {mode:o}); restrict it to the owner")]
    TooOpen { path: PathBuf, mode: u32 },
    #[error("cannot access the secret key file {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// The secret key; its bytes never appear in `Debug` output.
pub struct SecretKey([u8; SECRET_KEY_LEN]);

impl SecretKey {
    pub fn as_bytes(&self) -> &[u8; SECRET_KEY_LEN] {
        &self.0
    }
}

impl std::fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SecretKey(..)")
    }
}

fn io_error(path: &Path) -> impl FnOnce(std::io::Error) -> KeyFileError + '_ {
    move |source| KeyFileError::Io {
        path: path.to_path_buf(),
        source,
    }
}

/// Creates a new random key file with owner-only permissions. Never overwrites a file.
pub fn create(path: &Path) -> Result<SecretKey, KeyFileError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io_error(parent))?;
    }
    let mut key = [0u8; SECRET_KEY_LEN];
    OsRng.fill_bytes(&mut key);

    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(|source| {
        if source.kind() == ErrorKind::AlreadyExists {
            KeyFileError::AlreadyExists(path.to_path_buf())
        } else {
            io_error(path)(source)
        }
    })?;
    file.write_all(&key).map_err(io_error(path))?;
    file.sync_all().map_err(io_error(path))?;
    Ok(SecretKey(key))
}

/// Loads the key file, refusing a missing, malformed or group/world-readable file.
pub fn load(path: &Path) -> Result<SecretKey, KeyFileError> {
    let mut file = fs::File::open(path).map_err(|source| {
        if source.kind() == ErrorKind::NotFound {
            KeyFileError::Missing(path.to_path_buf())
        } else {
            io_error(path)(source)
        }
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = file
            .metadata()
            .map_err(io_error(path))?
            .permissions()
            .mode()
            & 0o777;
        if mode & 0o077 != 0 {
            return Err(KeyFileError::TooOpen {
                path: path.to_path_buf(),
                mode,
            });
        }
    }
    let mut bytes = Vec::with_capacity(SECRET_KEY_LEN);
    file.read_to_end(&mut bytes).map_err(io_error(path))?;
    let key: [u8; SECRET_KEY_LEN] =
        bytes
            .as_slice()
            .try_into()
            .map_err(|_| KeyFileError::WrongLength {
                path: path.to_path_buf(),
                found: bytes.len(),
            })?;
    Ok(SecretKey(key))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp() -> tempfile::TempDir {
        tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"))
    }

    #[test]
    fn create_then_load() {
        let dir = temp();
        let path = dir.path().join("keys").join("secret.key");
        let created = create(&path).unwrap_or_else(|e| panic!("{e}"));
        let loaded = load(&path).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(created.as_bytes(), loaded.as_bytes());
        assert_eq!(format!("{loaded:?}"), "SecretKey(..)");
    }

    #[test]
    fn never_overwrites() {
        let dir = temp();
        let path = dir.path().join("secret.key");
        assert!(create(&path).is_ok());
        assert!(matches!(create(&path), Err(KeyFileError::AlreadyExists(_))));
    }

    #[test]
    fn missing_file_names_init() {
        let dir = temp();
        let error = load(&dir.path().join("secret.key"));
        assert!(matches!(error, Err(KeyFileError::Missing(_))));
    }

    #[test]
    fn wrong_length_is_refused() {
        let dir = temp();
        let path = dir.path().join("secret.key");
        assert!(create(&path).is_ok());
        let mut file = OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap_or_else(|e| panic!("{e}"));
        file.write_all(b"x").unwrap_or_else(|e| panic!("{e}"));
        assert!(matches!(load(&path), Err(KeyFileError::WrongLength { .. })));
    }

    #[cfg(unix)]
    #[test]
    fn readable_by_others_is_refused() {
        use std::os::unix::fs::PermissionsExt;
        let dir = temp();
        let path = dir.path().join("secret.key");
        assert!(create(&path).is_ok());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644))
            .unwrap_or_else(|e| panic!("{e}"));
        assert!(matches!(load(&path), Err(KeyFileError::TooOpen { .. })));
    }
}
