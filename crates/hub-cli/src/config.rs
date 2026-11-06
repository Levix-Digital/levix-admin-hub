//! Where the CLI keeps its session: one file, readable by the user only (research R9).

use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{CliError, CliResult};

/// The signed-in Hub.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ca_file: Option<PathBuf>,
    pub email: String,
    pub token: String,
}

/// `LEVIX_HUB_CLI_CONFIG_DIR`, else the user's configuration directory: `%APPDATA%` on
/// Windows, `~/Library/Application Support` on macOS, `$XDG_CONFIG_HOME` or `~/.config`
/// elsewhere.
pub fn config_dir() -> CliResult<PathBuf> {
    let variable = |name: &str| {
        std::env::var_os(name)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    if let Some(dir) = variable("LEVIX_HUB_CLI_CONFIG_DIR") {
        return Ok(dir);
    }
    let base = if cfg!(windows) {
        variable("APPDATA")
    } else if cfg!(target_os = "macos") {
        variable("HOME").map(|home| home.join("Library/Application Support"))
    } else {
        variable("XDG_CONFIG_HOME").or_else(|| variable("HOME").map(|home| home.join(".config")))
    };
    base.map(|dir| dir.join("levix-hub")).ok_or_else(|| {
        CliError::Local("no configuration directory; set LEVIX_HUB_CLI_CONFIG_DIR".to_owned())
    })
}

fn session_path() -> CliResult<PathBuf> {
    Ok(config_dir()?.join("session.json"))
}

pub fn load() -> CliResult<Option<Session>> {
    let path = session_path()?;
    match std::fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).map(Some).map_err(|e| {
            CliError::Local(format!(
                "{} is unreadable ({e}); sign in again",
                path.display()
            ))
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

/// Writes the session atomically with owner-only permissions.
pub fn save(session: &Session) -> CliResult<PathBuf> {
    let dir = config_dir()?;
    create_private_dir(&dir)?;
    let path = dir.join("session.json");
    let temporary = dir.join("session.json.tmp");
    let text = serde_json::to_string_pretty(session)
        .map_err(|e| CliError::Local(format!("encoding the session: {e}")))?;
    {
        let mut file = private_file(&temporary)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
    }
    std::fs::rename(&temporary, &path)?;
    Ok(path)
}

pub fn remove() -> CliResult<()> {
    match std::fs::remove_file(session_path()?) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error.into()),
        _ => Ok(()),
    }
}

fn create_private_dir(dir: &Path) -> CliResult<()> {
    std::fs::create_dir_all(dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

fn private_file(path: &Path) -> CliResult<std::fs::File> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(file)
}
