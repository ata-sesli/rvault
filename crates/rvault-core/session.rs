use crate::config::Config;
use directories::ProjectDirs;
use rand::Rng;
use rand::distr::Alphanumeric;
use rand::rng;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::time::SystemTime;

const CURRENT_SESSION_FILE: &str = "current";
const BROWSER_SESSION_FILE: &str = "browser-current";

fn ensure_session_dir(session_dir: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(session_dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(session_dir, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

fn open_private_new(path: &Path) -> Result<fs::File, std::io::Error> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}

fn validate_session_token(token: &str) -> Result<(), String> {
    if token.len() == 48 && token.bytes().all(|byte| byte.is_ascii_alphanumeric()) {
        Ok(())
    } else {
        Err("Invalid session token.".to_string())
    }
}

fn start_session_at(session_dir: &Path, encryption_key: &[u8]) -> Result<String, std::io::Error> {
    ensure_session_dir(session_dir)?;
    loop {
        let session_token: String = rng()
            .sample_iter(&Alphanumeric)
            .take(48)
            .map(char::from)
            .collect();
        let session_file_path = session_dir.join(&session_token);
        match open_private_new(&session_file_path) {
            Ok(mut file) => {
                file.write_all(encryption_key)?;
                return Ok(session_token);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
}

fn write_session_pointer(session_dir: &Path, name: &str, token: &str) -> Result<(), String> {
    validate_session_token(token)?;
    ensure_session_dir(session_dir).map_err(|error| error.to_string())?;
    let destination = session_dir.join(name);
    let temporary = session_dir.join(format!(".{name}.{}.tmp", rand::random::<u64>()));
    let result = (|| {
        let mut file = open_private_new(&temporary).map_err(|error| error.to_string())?;
        file.write_all(token.as_bytes())
            .map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
        fs::rename(&temporary, destination).map_err(|error| error.to_string())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

/// Returns the path to the secure directory used for session files.
fn get_session_dir() -> Result<PathBuf, std::io::Error> {
    if let Some(proj_dirs) = ProjectDirs::from("io.github", "ata-sesli", "RVault") {
        // Use a runtime-specific directory if available, otherwise cache.
        let runtime_dir = proj_dirs
            .runtime_dir()
            .unwrap_or_else(|| proj_dirs.cache_dir());
        let session_dir = runtime_dir.join("sessions");

        ensure_session_dir(&session_dir)?;
        Ok(session_dir)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not find project directories",
        ))
    }
}

/// Creates a new session by caching the encryption key in a secure temp file.
/// Returns the new session token, which is the filename.
pub fn start_session(encryption_key: &[u8]) -> Result<String, std::io::Error> {
    let _ = end_session();
    let session_dir = get_session_dir()?;
    start_session_at(&session_dir, encryption_key)
}

/// Validates the current session token (from the env var) and returns the encryption key.
pub fn get_key_from_session() -> Result<Vec<u8>, String> {
    // 1. Load the configuration at runtime.
    let config = Config::new().map_err(|e| format!("Failed to load config: {}", e))?;

    // 2. Create the timeout duration from the config value.
    let session_timeout =
        Duration::from_secs(config.session_timeout.parse::<u64>().unwrap_or(15) * 60);
    let token = read_current()?;
    validate_session_token(&token)?;

    let session_dir =
        get_session_dir().map_err(|e| format!("Error accessing session directory: {}", e))?;

    let session_file_path = session_dir.join(&token);

    if session_file_path.exists() {
        // Get the file's metadata to check its timestamp
        let metadata = fs::metadata(&session_file_path)
            .map_err(|e| format!("Failed to read session metadata: {}", e))?;

        let modified_time = metadata
            .modified()
            .map_err(|e| format!("Failed to get session timestamp: {}", e))?;

        // Calculate how long ago the session file was created/modified
        let age = SystemTime::now()
            .duration_since(modified_time)
            .map_err(|_| "System clock error.".to_string())?;

        // Check if the session has expired
        if age > session_timeout {
            // If it's too old, delete the file and reject the request. ⏰
            fs::remove_file(&session_file_path)
                .map_err(|e| format!("Failed to clean up expired session: {}", e))?;
            return Err("Vault is locked. Your session has expired.".to_string());
        }

        // If not expired, read the key from the file
        fs::read(session_file_path).map_err(|e| format!("Failed to read session key: {}", e))
    } else {
        Err("Vault is locked. Invalid or expired session.".to_string())
    }
}

/// Ends the current session by deleting the session file.
pub fn end_session() -> Result<(), String> {
    let _ = end_browser_session();
    let token = read_current()?;
    validate_session_token(&token)?;
    let session_dir =
        get_session_dir().map_err(|e| format!("Error accessing session directory: {}", e))?;

    let session_file_path = session_dir.join(token);

    if session_file_path.exists() {
        fs::remove_file(session_file_path)
            .map_err(|e| format!("Failed to remove session file: {}", e))?;
    }
    let current_file_path = session_dir.join(CURRENT_SESSION_FILE);
    if current_file_path.exists() {
        let _ = fs::remove_file(current_file_path);
    }

    Ok(())
}
pub fn write_current(token: &str) -> Result<(), String> {
    let session_dir = get_session_dir().map_err(|error| error.to_string())?;
    write_session_pointer(&session_dir, CURRENT_SESSION_FILE, token)
}
pub fn read_current() -> Result<String, String> {
    let p = get_session_dir()
        .map_err(|e| e.to_string())?
        .join(CURRENT_SESSION_FILE);
    let token = std::fs::read_to_string(p)
        .map_err(|e| format!("No active session to lock: {e}"))?
        .trim()
        .to_string();
    validate_session_token(&token)?;
    Ok(token)
}

pub fn start_browser_session(token: &str) -> Result<(), String> {
    let session_dir = get_session_dir().map_err(|error| error.to_string())?;
    write_session_pointer(&session_dir, BROWSER_SESSION_FILE, token)
}

pub fn end_browser_session() -> Result<(), String> {
    let p = get_session_dir()
        .map_err(|e| e.to_string())?
        .join(BROWSER_SESSION_FILE);
    if p.exists() {
        std::fs::remove_file(p)
            .map_err(|e| format!("Failed to remove browser session token: {e}"))?;
    }
    Ok(())
}

pub fn browser_session_is_active() -> bool {
    get_key_from_browser_session().is_ok()
}

pub fn get_key_from_browser_session() -> Result<Vec<u8>, String> {
    let current_token = read_current()?;
    let browser_token = read_browser_current()?;
    if !browser_session_tokens_match(&current_token, &browser_token) {
        return Err("Vault is locked in the browser extension.".to_string());
    }
    get_key_from_session()
}

fn read_browser_current() -> Result<String, String> {
    let p = get_session_dir()
        .map_err(|e| e.to_string())?
        .join(BROWSER_SESSION_FILE);
    let token = std::fs::read_to_string(p)
        .map_err(|e| format!("No active browser session: {e}"))?
        .trim()
        .to_string();
    validate_session_token(&token)?;
    Ok(token)
}

fn browser_session_tokens_match(current_token: &str, browser_token: &str) -> bool {
    !current_token.is_empty() && current_token == browser_token
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary_session_root() -> PathBuf {
        std::env::temp_dir().join(format!("rvault-session-test-{}", rand::random::<u64>()))
    }

    #[test]
    fn browser_session_requires_matching_current_token() {
        assert!(browser_session_tokens_match(
            "current-token",
            "current-token"
        ));
        assert!(!browser_session_tokens_match(
            "current-token",
            "other-token"
        ));
        assert!(!browser_session_tokens_match("current-token", ""));
    }

    #[cfg(unix)]
    #[test]
    fn session_root_and_files_are_owner_only() {
        use std::os::unix::fs::PermissionsExt;

        let root = temporary_session_root();
        let token = start_session_at(&root, &[9_u8; 32]).unwrap();
        write_session_pointer(&root, CURRENT_SESSION_FILE, &token).unwrap();

        assert_eq!(fs::metadata(&root).unwrap().permissions().mode() & 0o777, 0o700);
        assert_eq!(
            fs::metadata(root.join(&token)).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(root.join(CURRENT_SESSION_FILE))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_malformed_session_tokens() {
        let invalid = [
            String::new(),
            "../outside".to_string(),
            "abc/def".to_string(),
            "a".repeat(47),
            "a".repeat(49),
        ];
        for token in invalid {
            assert!(validate_session_token(&token).is_err());
        }
        assert!(validate_session_token(&"a".repeat(48)).is_ok());
    }
}
