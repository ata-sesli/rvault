use crate::config::Config;
use directories::ProjectDirs;
use rand::Rng;
use rand::distr::Alphanumeric;
use rand::rng;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const CURRENT_SESSION_FILE: &str = "current";
const BROWSER_SESSION_FILE: &str = "browser-current";
const SESSION_METADATA_VERSION: u8 = 1;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SessionExpiration {
    version: u8,
    expires_at: u64,
}

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

fn session_expiration_path(session_dir: &Path, token: &str) -> PathBuf {
    session_dir.join(format!("{token}.expires"))
}

fn start_session_with_timeout_at(
    session_dir: &Path,
    encryption_key: &[u8],
    timeout_minutes: u64,
    now: u64,
) -> Result<Option<String>, std::io::Error> {
    if timeout_minutes == 0 {
        return Ok(None);
    }
    let timeout_seconds = timeout_minutes.checked_mul(60).ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "session timeout overflow")
    })?;
    let expires_at = now.checked_add(timeout_seconds).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "session expiration overflow",
        )
    })?;
    let token = start_session_at(session_dir, encryption_key)?;
    let key_path = session_dir.join(&token);
    let sidecar_path = session_expiration_path(session_dir, &token);
    let result = (|| {
        let mut sidecar = open_private_new(&sidecar_path)?;
        serde_json::to_writer(
            &mut sidecar,
            &SessionExpiration {
                version: SESSION_METADATA_VERSION,
                expires_at,
            },
        )
        .map_err(std::io::Error::other)?;
        sidecar.sync_all()
    })();
    if let Err(error) = result {
        let key_cleanup = fs::remove_file(&key_path);
        let sidecar_cleanup = fs::remove_file(&sidecar_path);
        let cleanup_error = key_cleanup
            .err()
            .filter(|error| error.kind() != std::io::ErrorKind::NotFound)
            .or_else(|| {
                sidecar_cleanup
                    .err()
                    .filter(|error| error.kind() != std::io::ErrorKind::NotFound)
            });
        return Err(match cleanup_error {
            Some(cleanup) => std::io::Error::new(
                error.kind(),
                format!("{error}; failed to roll back timed session: {cleanup}"),
            ),
            None => error,
        });
    }
    Ok(Some(token))
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

pub fn start_session_with_timeout(
    encryption_key: &[u8],
    timeout_minutes: u64,
) -> Result<Option<String>, std::io::Error> {
    if timeout_minutes == 0 {
        return Ok(None);
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| std::io::Error::other("system clock is before the Unix epoch"))?
        .as_secs();
    timeout_minutes.checked_mul(60).ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "session timeout overflow")
    })?;
    now.checked_add(timeout_minutes * 60).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "session expiration overflow",
        )
    })?;
    let _ = end_session();
    let session_dir = get_session_dir()?;
    start_session_with_timeout_at(&session_dir, encryption_key, timeout_minutes, now)
}

/// Validates the current session token (from the env var) and returns the encryption key.
pub fn get_key_from_session() -> Result<Vec<u8>, String> {
    let token = read_current()?;
    validate_session_token(&token)?;
    let session_dir =
        get_session_dir().map_err(|e| format!("Error accessing session directory: {}", e))?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "System clock error.".to_string())?
        .as_secs();
    let legacy_timeout = if session_expiration_path(&session_dir, &token).exists() {
        0
    } else {
        let config = Config::new().map_err(|e| format!("Failed to load config: {e}"))?;
        config.session_timeout.parse::<u64>().unwrap_or(15) * 60
    };
    get_key_from_session_at(&session_dir, &token, now, legacy_timeout)
}

fn get_key_from_session_at(
    session_dir: &Path,
    token: &str,
    now: u64,
    legacy_timeout_seconds: u64,
) -> Result<Vec<u8>, String> {
    validate_session_token(token)?;
    let key_path = session_dir.join(token);
    if !key_path.exists() {
        return Err("Vault is locked. Invalid or expired session.".to_string());
    }
    let sidecar_path = session_expiration_path(session_dir, token);
    if sidecar_path.exists() {
        let metadata = fs::read(&sidecar_path)
            .map_err(|error| format!("Invalid session metadata: {error}"))
            .and_then(|bytes| {
                serde_json::from_slice::<SessionExpiration>(&bytes)
                    .map_err(|error| format!("Invalid session metadata: {error}"))
            });
        match metadata {
            Ok(metadata)
                if metadata.version == SESSION_METADATA_VERSION && now < metadata.expires_at => {}
            Ok(metadata) if metadata.version != SESSION_METADATA_VERSION => {
                let cleanup = cleanup_session_files(session_dir, token);
                return Err(with_cleanup_error(
                    "Invalid session metadata version".to_string(),
                    cleanup,
                ));
            }
            Ok(_) => {
                let cleanup = cleanup_session_files(session_dir, token);
                return Err(with_cleanup_error(
                    "Vault is locked. Your session has expired.".to_string(),
                    cleanup,
                ));
            }
            Err(error) => {
                let cleanup = cleanup_session_files(session_dir, token);
                return Err(with_cleanup_error(error, cleanup));
            }
        }
    } else {
        let modified = fs::metadata(&key_path)
            .and_then(|metadata| metadata.modified())
            .and_then(|time| {
                time.duration_since(UNIX_EPOCH)
                    .map_err(std::io::Error::other)
            })
            .map_err(|error| format!("Failed to read session metadata: {error}"))?
            .as_secs();
        if now.saturating_sub(modified) > legacy_timeout_seconds {
            let cleanup = cleanup_session_files(session_dir, token);
            return Err(with_cleanup_error(
                "Vault is locked. Your session has expired.".to_string(),
                cleanup,
            ));
        }
    }
    fs::read(key_path).map_err(|error| format!("Failed to read session key: {error}"))
}

fn with_cleanup_error(message: String, cleanup: Result<(), String>) -> String {
    match cleanup {
        Ok(()) => message,
        Err(error) => format!("{message}; cleanup failed: {error}"),
    }
}

fn cleanup_session_files(session_dir: &Path, token: &str) -> Result<(), String> {
    let mut errors = Vec::new();
    for path in [
        session_dir.join(token),
        session_expiration_path(session_dir, token),
    ] {
        if let Err(error) = fs::remove_file(path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            errors.push(error.to_string());
        }
    }
    for pointer in [CURRENT_SESSION_FILE, BROWSER_SESSION_FILE] {
        let path = session_dir.join(pointer);
        match fs::read_to_string(&path) {
            Ok(value) if value.trim() == token => {
                if let Err(error) = fs::remove_file(path) {
                    errors.push(error.to_string());
                }
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => errors.push(error.to_string()),
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

/// Ends the current session by deleting the session file.
pub fn end_session() -> Result<(), String> {
    let _ = end_browser_session();
    let token = read_current()?;
    validate_session_token(&token)?;
    let session_dir =
        get_session_dir().map_err(|e| format!("Error accessing session directory: {}", e))?;

    cleanup_session_files(&session_dir, &token)
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

        assert_eq!(
            fs::metadata(&root).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(root.join(&token))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
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

    #[test]
    fn zero_timeout_is_a_filesystem_noop() {
        let root = temporary_session_root();
        assert_eq!(
            start_session_with_timeout_at(&root, &[7_u8; 32], 0, 1_000).unwrap(),
            None
        );
        assert!(!root.exists());
    }

    #[test]
    fn timed_session_writes_versioned_metadata_and_expires_absolutely() {
        let root = temporary_session_root();
        let token = start_session_with_timeout_at(&root, &[8_u8; 32], 2, 1_000)
            .unwrap()
            .unwrap();
        write_session_pointer(&root, CURRENT_SESSION_FILE, &token).unwrap();
        write_session_pointer(&root, BROWSER_SESSION_FILE, &token).unwrap();
        let sidecar = session_expiration_path(&root, &token);

        assert_eq!(fs::read(root.join(&token)).unwrap(), [8_u8; 32]);
        assert_eq!(
            fs::read_to_string(&sidecar).unwrap(),
            r#"{"version":1,"expires_at":1120}"#
        );
        assert_eq!(
            get_key_from_session_at(&root, &token, 1_119, 900).unwrap(),
            [8_u8; 32]
        );
        assert!(get_key_from_session_at(&root, &token, 1_120, 900).is_err());
        assert!(!root.join(&token).exists());
        assert!(!sidecar.exists());
        assert!(!root.join(CURRENT_SESSION_FILE).exists());
        assert!(!root.join(BROWSER_SESSION_FILE).exists());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn malformed_timed_metadata_is_rejected_without_legacy_fallback() {
        let root = temporary_session_root();
        let token = start_session_at(&root, &[5_u8; 32]).unwrap();
        fs::write(session_expiration_path(&root, &token), b"{}").unwrap();
        write_session_pointer(&root, CURRENT_SESSION_FILE, &token).unwrap();

        let error = get_key_from_session_at(&root, &token, 1_000, u64::MAX).unwrap_err();

        assert!(error.contains("metadata"));
        assert!(!root.join(&token).exists());
        assert!(!root.join(CURRENT_SESSION_FILE).exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn expired_cleanup_preserves_pointers_for_a_newer_token() {
        let root = temporary_session_root();
        let expired = start_session_with_timeout_at(&root, &[1_u8; 32], 1, 1_000)
            .unwrap()
            .unwrap();
        let newer = start_session_at(&root, &[2_u8; 32]).unwrap();
        write_session_pointer(&root, CURRENT_SESSION_FILE, &newer).unwrap();
        write_session_pointer(&root, BROWSER_SESSION_FILE, &newer).unwrap();

        assert!(get_key_from_session_at(&root, &expired, 1_060, 900).is_err());
        assert_eq!(
            fs::read_to_string(root.join(CURRENT_SESSION_FILE)).unwrap(),
            newer
        );
        assert_eq!(
            fs::read_to_string(root.join(BROWSER_SESSION_FILE)).unwrap(),
            newer
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn timed_session_sidecar_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;

        let root = temporary_session_root();
        let token = start_session_with_timeout_at(&root, &[3_u8; 32], 1, 1_000)
            .unwrap()
            .unwrap();
        assert_eq!(
            fs::metadata(session_expiration_path(&root, &token))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        fs::remove_dir_all(root).unwrap();
    }
}
