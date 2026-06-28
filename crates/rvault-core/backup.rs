use crate::{
    binary::{BACKUP_MAGIC, decode_envelope, encode_envelope},
    config::config_path,
    crypto::{decrypt_bytes_with_key, derive_key, encrypt_bytes_with_key},
    identity::identity_path,
    keystore::keystore_path,
    storage::database_path,
};
use chrono::Utc;
use rand::RngCore;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const BACKUP_SALT_LEN: usize = 16;
const BACKUP_PAYLOAD_MAGIC: &[u8; 8] = b"RVBKPAY1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupPayload {
    pub created_at: i64,
    pub config: Vec<u8>,
    pub keystore: Vec<u8>,
    pub database: Vec<u8>,
    pub identity: Option<Vec<u8>>,
}

pub fn create_backup_bytes(
    master_password: &str,
    payload: &BackupPayload,
) -> Result<Vec<u8>, String> {
    let mut salt = [0_u8; BACKUP_SALT_LEN];
    rand::rng().fill_bytes(&mut salt);
    let key = derive_key(master_password.as_bytes(), &salt).map_err(|e| e.to_string())?;
    let payload_bytes = encode_backup_payload(payload)?;
    let (nonce, ciphertext) = encrypt_bytes_with_key(&key, &payload_bytes)?;
    Ok(encode_envelope(
        BACKUP_MAGIC,
        &[salt.to_vec(), nonce.to_vec(), ciphertext],
    ))
}

pub fn decrypt_backup_bytes(master_password: &str, bytes: &[u8]) -> Result<BackupPayload, String> {
    let envelope = decode_envelope(bytes, BACKUP_MAGIC)?;
    if envelope.fields.len() != 3 {
        return Err("invalid backup envelope field count".to_string());
    }
    let salt = &envelope.fields[0];
    if salt.len() != BACKUP_SALT_LEN {
        return Err("invalid backup salt length".to_string());
    }
    let key = derive_key(master_password.as_bytes(), salt).map_err(|e| e.to_string())?;
    let payload_bytes = decrypt_bytes_with_key(&key, &envelope.fields[1], &envelope.fields[2])?;
    decode_backup_payload(&payload_bytes)
}

pub fn validate_backup_envelope(bytes: &[u8]) -> Result<(), String> {
    decode_envelope(bytes, BACKUP_MAGIC).map(|_| ())
}

pub fn create_backup_file(master_password: &str, out_path: &Path) -> Result<(), String> {
    let _ = crate::storage::Database::new().map_err(|e| format!("open database: {e}"))?;
    let payload = BackupPayload {
        created_at: Utc::now().timestamp(),
        config: fs::read(config_path().map_err(|e| e.to_string())?)
            .map_err(|e| format!("read config: {e}"))?,
        keystore: fs::read(keystore_path()?).map_err(|e| format!("read keystore: {e}"))?,
        database: fs::read(database_path().map_err(|e| e.to_string())?)
            .map_err(|e| format!("read database: {e}"))?,
        identity: match identity_path() {
            Ok(path) if path.exists() => {
                Some(fs::read(path).map_err(|e| format!("read identity: {e}"))?)
            }
            _ => None,
        },
    };
    let bytes = create_backup_bytes(master_password, &payload)?;
    write_atomic(out_path, &bytes)
}

pub fn restore_backup_file(master_password: &str, backup_path: &Path) -> Result<(), String> {
    let bytes = fs::read(backup_path).map_err(|e| format!("read backup: {e}"))?;
    let payload = decrypt_backup_bytes(master_password, &bytes)?;
    let targets = RestoreTargets {
        config: config_path().map_err(|e| e.to_string())?,
        keystore: keystore_path()?,
        database: database_path().map_err(|e| e.to_string())?,
        identity: identity_path()?,
    };
    restore_payload_to_targets(&payload, &targets)
}

#[derive(Debug)]
struct RestoreTargets {
    config: PathBuf,
    keystore: PathBuf,
    database: PathBuf,
    identity: PathBuf,
}

#[derive(Debug)]
struct RestoreOp {
    target: PathBuf,
    staged: Option<PathBuf>,
    backup: PathBuf,
    target_existed: bool,
    backup_created: bool,
    target_written: bool,
}

fn restore_payload_to_targets(
    payload: &BackupPayload,
    targets: &RestoreTargets,
) -> Result<(), String> {
    let suffix = restore_temp_suffix();
    let restore_items = vec![
        (targets.config.clone(), Some(payload.config.as_slice())),
        (targets.keystore.clone(), Some(payload.keystore.as_slice())),
        (targets.database.clone(), Some(payload.database.as_slice())),
        (targets.identity.clone(), payload.identity.as_deref()),
    ];
    let mut ops = Vec::with_capacity(restore_items.len());

    for (target, bytes) in restore_items {
        match stage_restore_op(target, bytes, &suffix) {
            Ok(op) => ops.push(op),
            Err(e) => {
                cleanup_restore_files(&ops);
                return Err(e);
            }
        }
    }

    if let Err(commit_error) = commit_restore_ops(&mut ops) {
        let rollback_result = rollback_restore_ops(&mut ops);
        cleanup_restore_files(&ops);
        if let Err(rollback_error) = rollback_result {
            return Err(format!("{commit_error}; rollback failed: {rollback_error}"));
        }
        return Err(commit_error);
    }

    cleanup_restore_files(&ops);
    Ok(())
}

fn stage_restore_op(
    target: PathBuf,
    bytes: Option<&[u8]>,
    suffix: &str,
) -> Result<RestoreOp, String> {
    let staged = if let Some(bytes) = bytes {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
        }
        let staged = restore_temp_path_for(&target, suffix, "stage");
        fs::write(&staged, bytes).map_err(|e| format!("write {}: {e}", staged.display()))?;
        Some(staged)
    } else {
        None
    };

    let target_existed = target
        .try_exists()
        .map_err(|e| format!("check {}: {e}", target.display()))?;

    Ok(RestoreOp {
        backup: restore_temp_path_for(&target, suffix, "backup"),
        target,
        staged,
        target_existed,
        backup_created: false,
        target_written: false,
    })
}

fn commit_restore_ops(ops: &mut [RestoreOp]) -> Result<(), String> {
    for op in ops {
        if op.target_existed {
            fs::rename(&op.target, &op.backup)
                .map_err(|e| format!("backup {}: {e}", op.target.display()))?;
            op.backup_created = true;
        }

        if let Some(staged) = &op.staged {
            fs::rename(staged, &op.target)
                .map_err(|e| format!("replace {}: {e}", op.target.display()))?;
            op.target_written = true;
        }
    }
    Ok(())
}

fn rollback_restore_ops(ops: &mut [RestoreOp]) -> Result<(), String> {
    let mut errors = Vec::new();
    for op in ops.iter_mut().rev() {
        if op.target_written {
            if let Err(e) = fs::remove_file(&op.target) {
                errors.push(format!("remove restored {}: {e}", op.target.display()));
            }
            op.target_written = false;
        }

        if op.backup_created {
            if let Err(e) = fs::rename(&op.backup, &op.target) {
                errors.push(format!("restore original {}: {e}", op.target.display()));
            }
            op.backup_created = false;
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

fn cleanup_restore_files(ops: &[RestoreOp]) {
    for op in ops {
        if let Some(staged) = &op.staged {
            let _ = fs::remove_file(staged);
        }
        let _ = fs::remove_file(&op.backup);
    }
}

fn restore_temp_path_for(path: &Path, suffix: &str, kind: &str) -> PathBuf {
    let mut tmp = path.as_os_str().to_os_string();
    tmp.push(format!(".{kind}{suffix}"));
    PathBuf::from(tmp)
}

fn restore_temp_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!(".{}-{nanos}", std::process::id())
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    let tmp_path = temp_path_for(path);
    fs::write(&tmp_path, bytes).map_err(|e| format!("write {}: {e}", tmp_path.display()))?;
    fs::rename(&tmp_path, path).map_err(|e| format!("replace {}: {e}", path.display()))
}

fn temp_path_for(path: &Path) -> PathBuf {
    let mut tmp = path.as_os_str().to_os_string();
    tmp.push(".tmp");
    PathBuf::from(tmp)
}

fn encode_backup_payload(payload: &BackupPayload) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    out.extend_from_slice(BACKUP_PAYLOAD_MAGIC);
    out.extend_from_slice(&payload.created_at.to_le_bytes());
    push_blob(&mut out, &payload.config)?;
    push_blob(&mut out, &payload.keystore)?;
    push_blob(&mut out, &payload.database)?;
    push_blob(&mut out, payload.identity.as_deref().unwrap_or(&[]))?;
    Ok(out)
}

fn decode_backup_payload(bytes: &[u8]) -> Result<BackupPayload, String> {
    if bytes.len() < BACKUP_PAYLOAD_MAGIC.len() + 8 {
        return Err("truncated backup payload".to_string());
    }
    if &bytes[..BACKUP_PAYLOAD_MAGIC.len()] != BACKUP_PAYLOAD_MAGIC {
        return Err("invalid backup payload magic".to_string());
    }
    let mut cursor = BACKUP_PAYLOAD_MAGIC.len();
    let created_at = read_i64(bytes, &mut cursor)?;
    let config = read_blob(bytes, &mut cursor)?;
    let keystore = read_blob(bytes, &mut cursor)?;
    let database = read_blob(bytes, &mut cursor)?;
    let identity = read_blob(bytes, &mut cursor)?;
    if cursor != bytes.len() {
        return Err("backup payload has trailing bytes".to_string());
    }
    Ok(BackupPayload {
        created_at,
        config,
        keystore,
        database,
        identity: (!identity.is_empty()).then_some(identity),
    })
}

fn push_blob(out: &mut Vec<u8>, bytes: &[u8]) -> Result<(), String> {
    let len: u64 = bytes
        .len()
        .try_into()
        .map_err(|_| "backup payload blob is too large".to_string())?;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(bytes);
    Ok(())
}

fn read_i64(bytes: &[u8], cursor: &mut usize) -> Result<i64, String> {
    if bytes.len().saturating_sub(*cursor) < 8 {
        return Err("truncated backup payload timestamp".to_string());
    }
    let mut out = [0_u8; 8];
    out.copy_from_slice(&bytes[*cursor..*cursor + 8]);
    *cursor += 8;
    Ok(i64::from_le_bytes(out))
}

fn read_blob(bytes: &[u8], cursor: &mut usize) -> Result<Vec<u8>, String> {
    if bytes.len().saturating_sub(*cursor) < 8 {
        return Err("truncated backup payload blob length".to_string());
    }
    let mut len_bytes = [0_u8; 8];
    len_bytes.copy_from_slice(&bytes[*cursor..*cursor + 8]);
    *cursor += 8;
    let len = u64::from_le_bytes(len_bytes) as usize;
    if bytes.len().saturating_sub(*cursor) < len {
        return Err("truncated backup payload blob".to_string());
    }
    let out = bytes[*cursor..*cursor + len].to_vec();
    *cursor += len;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload() -> BackupPayload {
        BackupPayload {
            created_at: 1_719_000_000,
            config: b"config bytes".to_vec(),
            keystore: b"keystore bytes".to_vec(),
            database: b"database bytes".to_vec(),
            identity: Some(b"identity bytes".to_vec()),
        }
    }

    #[test]
    fn backup_bytes_round_trip_as_encrypted_binary_envelope() {
        let backup =
            create_backup_bytes("correct horse battery staple", &payload()).expect("create backup");

        assert!(backup.starts_with(BACKUP_MAGIC));
        assert!(!String::from_utf8_lossy(&backup).contains("config bytes"));
        assert!(!String::from_utf8_lossy(&backup).contains("database bytes"));

        let restored =
            decrypt_backup_bytes("correct horse battery staple", &backup).expect("decrypt backup");

        assert_eq!(restored, payload());
    }

    #[test]
    fn backup_bytes_reject_wrong_password() {
        let backup =
            create_backup_bytes("correct horse battery staple", &payload()).expect("create backup");

        let err = decrypt_backup_bytes("wrong password", &backup)
            .expect_err("wrong password should fail");

        assert!(err.contains("decrypt"));
    }

    #[test]
    fn backup_envelope_rejects_wrong_magic() {
        let bytes = encode_envelope(crate::binary::EXPORT_MAGIC, &[b"payload".to_vec()]);

        let err = validate_backup_envelope(&bytes).expect_err("wrong magic should fail");

        assert!(err.contains("magic"));
    }

    #[test]
    fn restore_payload_does_not_replace_existing_files_when_staging_fails() {
        let root =
            std::env::temp_dir().join(format!("rvault-backup-restore-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create test root");

        let config = root.join("config.json");
        let keystore = root.join("keystore.rvault");
        let database = root.join("blocked").join("default.sqlite");
        let identity = root.join("identity.rvault");
        fs::write(&config, b"original config").expect("write config");
        fs::write(&keystore, b"original keystore").expect("write keystore");
        fs::write(&identity, b"original identity").expect("write identity");
        fs::write(root.join("blocked"), b"not a directory").expect("write blocker");

        let targets = RestoreTargets {
            config: config.clone(),
            keystore: keystore.clone(),
            database,
            identity: identity.clone(),
        };

        let err =
            restore_payload_to_targets(&payload(), &targets).expect_err("staging should fail");

        assert!(err.contains("mkdir") || err.contains("write"));
        assert_eq!(fs::read(&config).expect("read config"), b"original config");
        assert_eq!(
            fs::read(&keystore).expect("read keystore"),
            b"original keystore"
        );
        assert_eq!(
            fs::read(&identity).expect("read identity"),
            b"original identity"
        );

        let _ = fs::remove_dir_all(&root);
    }
}
