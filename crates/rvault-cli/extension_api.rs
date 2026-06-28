use base64::{Engine as _, engine::general_purpose::STANDARD as Base64};
use rvault_core::{
    backup,
    config::Config,
    crypto, identity,
    portable_export::{self, ExportEntry},
    session,
    storage::{Database, Table},
    vault::Vault,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

const TRANSFER_CHUNK_SIZE: usize = 512 * 1024;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum HostRequest {
    #[serde(rename = "status")]
    Status,
    #[serde(rename = "unlock")]
    Unlock {
        #[serde(rename = "masterPassword")]
        master_password: String,
    },
    #[serde(rename = "lock")]
    Lock,
    #[serde(rename = "quit")]
    Quit,
    #[serde(rename = "list")]
    List {
        query: Option<String>,
        vault: Option<String>,
    },
    #[serde(rename = "get")]
    Get {
        platform: String,
        #[serde(rename = "userId")]
        user_id: String,
        vault: Option<String>,
    },
    #[serde(rename = "create")]
    Create {
        platform: String,
        #[serde(rename = "userId")]
        user_id: String,
        password: String,
        vault: Option<String>,
    },
    #[serde(rename = "update")]
    Update {
        platform: String,
        #[serde(rename = "oldUserId")]
        old_user_id: String,
        #[serde(rename = "newUserId")]
        new_user_id: String,
        password: String,
        vault: Option<String>,
    },
    #[serde(rename = "delete")]
    Delete {
        platform: String,
        #[serde(rename = "userId")]
        user_id: String,
        vault: Option<String>,
    },
    #[serde(rename = "generate")]
    Generate {
        length: u8,
        #[serde(rename = "specialCharacters")]
        special_characters: bool,
    },
    #[serde(rename = "identity")]
    Identity,
    #[serde(rename = "backupCreate")]
    BackupCreate {
        #[serde(rename = "masterPassword")]
        master_password: String,
    },
    #[serde(rename = "backupRestore")]
    BackupRestore {
        #[serde(rename = "masterPassword")]
        master_password: String,
        token: String,
    },
    #[serde(rename = "export")]
    Export {
        to: String,
        entries: Vec<HostEntrySelector>,
        vault: Option<String>,
    },
    #[serde(rename = "importPreview")]
    ImportPreview {
        token: String,
        vault: Option<String>,
    },
    #[serde(rename = "importApply")]
    ImportApply {
        token: String,
        vault: Option<String>,
        #[serde(rename = "overwriteAll")]
        overwrite_all: Option<bool>,
        #[serde(rename = "skipAll")]
        skip_all: Option<bool>,
        decisions: Option<Vec<HostImportDecision>>,
    },
    #[serde(rename = "downloadChunk")]
    DownloadChunk {
        token: String,
        offset: u64,
        length: Option<usize>,
    },
    #[serde(rename = "uploadStart")]
    UploadStart,
    #[serde(rename = "uploadChunk")]
    UploadChunk {
        token: String,
        #[serde(rename = "contentBase64")]
        content_base64: String,
    },
    #[serde(rename = "transferFinish")]
    TransferFinish { token: String },
}

#[derive(Debug, Deserialize)]
struct HostEntrySelector {
    platform: String,
    #[serde(rename = "userId")]
    user_id: String,
}

#[derive(Debug, Deserialize)]
struct HostImportDecision {
    platform: String,
    #[serde(rename = "userId")]
    user_id: String,
    action: String,
}

#[derive(Serialize)]
struct HostError {
    code: &'static str,
    message: String,
}

pub fn handle_request_json(input: &str) -> String {
    let request = match serde_json::from_str::<HostRequest>(input) {
        Ok(request) => request,
        Err(e) => return error("invalid_request", format!("invalid request: {e}")),
    };

    match handle_request(request) {
        Ok(data) => ok(data),
        Err(response) => response,
    }
}

fn handle_request(request: HostRequest) -> Result<Value, String> {
    match request {
        HostRequest::Status => status(),
        HostRequest::Unlock { master_password } => unlock(master_password),
        HostRequest::Lock => {
            let _ = session::end_session();
            Ok(json!({ "locked": true }))
        }
        HostRequest::Quit => {
            let _ = session::end_browser_session();
            Ok(json!({ "locked": true }))
        }
        HostRequest::List { query, vault } => with_unlocked_table(vault, |db, table, _key| {
            let normalized_query = query.unwrap_or_default().to_lowercase();
            let entries = table
                .list(db)
                .map_err(|e| storage_error(e.to_string()))?
                .into_iter()
                .filter(|entry| {
                    normalized_query.is_empty()
                        || entry.platform.to_lowercase().contains(&normalized_query)
                        || entry.user_id.to_lowercase().contains(&normalized_query)
                })
                .map(|entry| {
                    json!({
                        "platform": entry.platform,
                        "userId": entry.user_id,
                        "pinned": entry.pinned,
                        "createdAt": entry.created_at,
                        "updatedAt": entry.updated_at,
                    })
                })
                .collect::<Vec<_>>();
            Ok(json!({ "entries": entries }))
        }),
        HostRequest::Get {
            platform,
            user_id,
            vault,
        } => with_unlocked_table(vault, |db, table, key| {
            let password = table
                .retrieve_password_with_key(db, key, platform, user_id)
                .map_err(|e| not_found_or_storage(e.to_string()))?;
            Ok(json!({ "password": password }))
        }),
        HostRequest::Create {
            platform,
            user_id,
            password,
            vault,
        } => with_unlocked_table(vault, |db, table, key| {
            table
                .add_entry_with_key_result(db, key, platform, user_id, password)
                .map_err(|e| storage_error(e.to_string()))?;
            Ok(json!({ "saved": true }))
        }),
        HostRequest::Update {
            platform,
            old_user_id,
            new_user_id,
            password,
            vault,
        } => with_unlocked_table(vault, |db, table, key| {
            table
                .update_entry(db, key, &platform, &old_user_id, &new_user_id, &password)
                .map_err(|e| storage_error(e.to_string()))?;
            Ok(json!({ "saved": true }))
        }),
        HostRequest::Delete {
            platform,
            user_id,
            vault,
        } => with_unlocked_table(vault, |db, table, _key| {
            table
                .remove_entry_result(db, platform, user_id)
                .map_err(|e| not_found_or_storage(e.to_string()))?;
            Ok(json!({ "deleted": true }))
        }),
        HostRequest::Generate {
            length,
            special_characters,
        } => {
            let min_length = if special_characters { 4 } else { 3 };
            let length = length.clamp(min_length, 128);
            Ok(json!({
                "password": crypto::generate_password(length, special_characters)
            }))
        }
        HostRequest::Identity => with_browser_key(|key| {
            let identity = identity::load_or_create_identity(key).map_err(storage_error)?;
            Ok(json!({ "publicCode": identity::public_code_from_key(&identity.public_key) }))
        }),
        HostRequest::BackupCreate { master_password } => create_backup_transfer(master_password),
        HostRequest::BackupRestore {
            master_password,
            token,
        } => {
            let path = transfer_path(&token)?;
            backup::restore_backup_file(&master_password, &path).map_err(storage_error)?;
            Ok(json!({ "restored": true }))
        }
        HostRequest::Export { to, entries, vault } => {
            with_unlocked_table(vault, |db, table, key| {
                let export_entries = build_export_entries(db, table, key, &entries)?;
                let bytes = portable_export::create_export_bytes(&to, &export_entries)
                    .map_err(storage_error)?;
                let token = write_transfer_file(&bytes)?;
                Ok(json!({
                    "token": token,
                    "size": bytes.len(),
                    "fileName": "rvault-export.rvault-export",
                    "chunkSize": TRANSFER_CHUNK_SIZE,
                }))
            })
        }
        HostRequest::ImportPreview { token, vault } => {
            with_unlocked_table(vault, |db, table, key| {
                let entries = decrypt_transfer_export(key, &token)?;
                let conflicts = import_conflicts(db, table, &entries)?;
                Ok(json!({
                    "entries": entries.iter().map(entry_metadata_json).collect::<Vec<_>>(),
                    "conflicts": conflicts,
                }))
            })
        }
        HostRequest::ImportApply {
            token,
            vault,
            overwrite_all,
            skip_all,
            decisions,
        } => with_unlocked_table(vault, |db, table, key| {
            apply_import(
                db,
                table,
                key,
                &token,
                overwrite_all.unwrap_or(false),
                skip_all.unwrap_or(false),
                decisions.unwrap_or_default(),
            )
        }),
        HostRequest::DownloadChunk {
            token,
            offset,
            length,
        } => download_chunk(&token, offset, length.unwrap_or(TRANSFER_CHUNK_SIZE)),
        HostRequest::UploadStart => {
            let token = create_empty_transfer_file()?;
            Ok(json!({ "token": token }))
        }
        HostRequest::UploadChunk {
            token,
            content_base64,
        } => {
            append_upload_chunk(&token, &content_base64)?;
            let size = fs::metadata(transfer_path(&token)?)
                .map_err(|e| storage_error(e.to_string()))?
                .len();
            Ok(json!({ "uploaded": size }))
        }
        HostRequest::TransferFinish { token } => {
            let path = transfer_path(&token)?;
            if path.exists() {
                fs::remove_file(&path).map_err(|e| storage_error(e.to_string()))?;
            }
            Ok(json!({ "removed": true }))
        }
    }
}

fn status() -> Result<Value, String> {
    let config = Config::new().map_err(|e| storage_error(e.to_string()))?;
    if config.master_password_hash.is_none() {
        return Err(error("setup_required", "RVault has not been set up."));
    }
    Ok(json!({
        "setupRequired": false,
        "locked": !session::browser_session_is_active(),
    }))
}

fn unlock(master_password: String) -> Result<Value, String> {
    let config = Config::new().map_err(|e| storage_error(e.to_string()))?;
    let Some(stored_hash) = config.master_password_hash.as_deref() else {
        return Err(error("setup_required", "RVault has not been set up."));
    };
    let encryption_key = Vault::get_encryption_key(&master_password, stored_hash)
        .map_err(|e| error("unlock_failed", e))?;
    let token =
        session::start_session(&encryption_key).map_err(|e| storage_error(e.to_string()))?;
    session::write_current(&token).map_err(storage_error)?;
    session::start_browser_session(&token).map_err(storage_error)?;
    Ok(json!({ "locked": false }))
}

fn with_browser_key<F>(operation: F) -> Result<Value, String>
where
    F: FnOnce(&[u8]) -> Result<Value, String>,
{
    let key = session::get_key_from_browser_session().map_err(|e| error("locked", e))?;
    operation(&key)
}

fn with_unlocked_table<F>(vault: Option<String>, operation: F) -> Result<Value, String>
where
    F: FnOnce(&Database, &Table, &[u8]) -> Result<Value, String>,
{
    let key = session::get_key_from_browser_session().map_err(|e| error("locked", e))?;
    let db = Database::new().map_err(|e| storage_error(e.to_string()))?;
    let table = Table::new(&db, vault).map_err(|e| storage_error(e.to_string()))?;
    operation(&db, &table, &key)
}

fn create_backup_transfer(master_password: String) -> Result<Value, String> {
    let config = Config::new().map_err(|e| storage_error(e.to_string()))?;
    let Some(stored_hash) = config.master_password_hash.as_deref() else {
        return Err(error("setup_required", "RVault has not been set up."));
    };
    let _ = Vault::get_encryption_key(&master_password, stored_hash)
        .map_err(|e| error("unlock_failed", e))?;
    let token = new_transfer_token();
    let path = transfer_path(&token)?;
    backup::create_backup_file(&master_password, &path).map_err(storage_error)?;
    let size = fs::metadata(&path)
        .map_err(|e| storage_error(e.to_string()))?
        .len();
    Ok(json!({
        "token": token,
        "size": size,
        "fileName": "rvault.rvault-backup",
        "chunkSize": TRANSFER_CHUNK_SIZE,
    }))
}

fn build_export_entries(
    db: &Database,
    table: &Table,
    key: &[u8],
    selectors: &[HostEntrySelector],
) -> Result<Vec<ExportEntry>, String> {
    if selectors.is_empty() {
        return Err(error("invalid_request", "No entries were selected."));
    }
    let metadata = table.list(db).map_err(|e| storage_error(e.to_string()))?;
    selectors
        .iter()
        .map(|selector| {
            let password = table
                .retrieve_password_with_key(
                    db,
                    key,
                    selector.platform.clone(),
                    selector.user_id.clone(),
                )
                .map_err(|e| not_found_or_storage(e.to_string()))?;
            let meta = metadata.iter().find(|entry| {
                entry.platform == selector.platform && entry.user_id == selector.user_id
            });
            Ok(ExportEntry {
                platform: selector.platform.clone(),
                user_id: selector.user_id.clone(),
                password,
                pinned: meta.map(|entry| entry.pinned).unwrap_or(false),
                created_at: meta.map(|entry| entry.created_at).unwrap_or(0),
                updated_at: meta.map(|entry| entry.updated_at).unwrap_or(0),
            })
        })
        .collect()
}

fn decrypt_transfer_export(key: &[u8], token: &str) -> Result<Vec<ExportEntry>, String> {
    let identity = identity::load_or_create_identity(key).map_err(storage_error)?;
    let bytes = fs::read(transfer_path(token)?).map_err(|e| storage_error(e.to_string()))?;
    portable_export::decrypt_export_bytes(&identity, &bytes).map_err(storage_error)
}

fn import_conflicts(
    db: &Database,
    table: &Table,
    entries: &[ExportEntry],
) -> Result<Vec<Value>, String> {
    entries
        .iter()
        .filter_map(
            |entry| match table.entry_exists(db, &entry.platform, &entry.user_id) {
                Ok(true) => Some(Ok(json!({
                    "platform": entry.platform,
                    "userId": entry.user_id,
                }))),
                Ok(false) => None,
                Err(e) => Some(Err(storage_error(e.to_string()))),
            },
        )
        .collect()
}

fn apply_import(
    db: &Database,
    table: &Table,
    key: &[u8],
    token: &str,
    overwrite_all: bool,
    skip_all: bool,
    decisions: Vec<HostImportDecision>,
) -> Result<Value, String> {
    if overwrite_all && skip_all {
        return Err(error(
            "invalid_request",
            "overwriteAll and skipAll cannot both be true.",
        ));
    }
    let entries = decrypt_transfer_export(key, token)?;
    let mut imported = 0;
    let mut skipped = 0;
    for entry in entries {
        let exists = table
            .entry_exists(db, &entry.platform, &entry.user_id)
            .map_err(|e| storage_error(e.to_string()))?;
        let should_import = if exists {
            if skip_all {
                false
            } else if overwrite_all {
                true
            } else {
                match decisions.iter().find(|decision| {
                    decision.platform == entry.platform && decision.user_id == entry.user_id
                }) {
                    Some(decision) if decision.action == "overwrite" => true,
                    Some(decision) if decision.action == "skip" => false,
                    _ => {
                        return Err(error(
                            "invalid_request",
                            format!(
                                "Missing import decision for {} / {}.",
                                entry.platform, entry.user_id
                            ),
                        ));
                    }
                }
            }
        } else {
            true
        };
        if should_import {
            table
                .import_entry_with_key_result(db, key, &entry)
                .map_err(|e| storage_error(e.to_string()))?;
            imported += 1;
        } else {
            skipped += 1;
        }
    }
    Ok(json!({ "imported": imported, "skipped": skipped }))
}

fn entry_metadata_json(entry: &ExportEntry) -> Value {
    json!({
        "platform": entry.platform,
        "userId": entry.user_id,
        "pinned": entry.pinned,
        "createdAt": entry.created_at,
        "updatedAt": entry.updated_at,
    })
}

fn download_chunk(token: &str, offset: u64, length: usize) -> Result<Value, String> {
    let path = transfer_path(token)?;
    let mut file = fs::File::open(&path).map_err(|e| storage_error(e.to_string()))?;
    let size = file
        .metadata()
        .map_err(|e| storage_error(e.to_string()))?
        .len();
    file.seek(SeekFrom::Start(offset))
        .map_err(|e| storage_error(e.to_string()))?;
    let mut buffer = vec![0_u8; length.min(TRANSFER_CHUNK_SIZE)];
    let read = file
        .read(&mut buffer)
        .map_err(|e| storage_error(e.to_string()))?;
    buffer.truncate(read);
    Ok(json!({
        "contentBase64": Base64.encode(&buffer),
        "offset": offset,
        "nextOffset": offset + read as u64,
        "size": size,
        "done": offset + read as u64 >= size,
    }))
}

fn create_empty_transfer_file() -> Result<String, String> {
    let token = new_transfer_token();
    let path = transfer_path(&token)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| storage_error(e.to_string()))?;
    }
    fs::File::create(&path).map_err(|e| storage_error(e.to_string()))?;
    Ok(token)
}

fn write_transfer_file(bytes: &[u8]) -> Result<String, String> {
    let token = create_empty_transfer_file()?;
    fs::write(transfer_path(&token)?, bytes).map_err(|e| storage_error(e.to_string()))?;
    Ok(token)
}

fn append_upload_chunk(token: &str, content_base64: &str) -> Result<(), String> {
    let bytes = Base64
        .decode(content_base64)
        .map_err(|e| error("invalid_request", format!("invalid upload chunk: {e}")))?;
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(transfer_path(token)?)
        .map_err(|e| storage_error(e.to_string()))?;
    file.write_all(&bytes)
        .map_err(|e| storage_error(e.to_string()))
}

fn transfer_path(token: &str) -> Result<PathBuf, String> {
    if token.is_empty()
        || !token
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(error("invalid_request", "Invalid transfer token."));
    }
    let dir = std::env::temp_dir().join("rvault-transfers");
    fs::create_dir_all(&dir).map_err(|e| storage_error(e.to_string()))?;
    Ok(dir.join(token))
}

fn new_transfer_token() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{}-{nanos}", std::process::id())
}

fn ok(data: Value) -> String {
    json!({ "ok": true, "data": data }).to_string()
}

fn error(code: &'static str, message: impl Into<String>) -> String {
    json!({
        "ok": false,
        "error": HostError {
            code,
            message: message.into(),
        }
    })
    .to_string()
}

fn storage_error(message: impl Into<String>) -> String {
    error("storage_error", message)
}

fn not_found_or_storage(message: String) -> String {
    if message.contains("Query returned no rows") || message.contains("QueryReturnedNoRows") {
        error("not_found", "No matching entry found.")
    } else {
        storage_error(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_request_returns_password_without_requiring_session() {
        let response =
            handle_request_json(r#"{"type":"generate","length":16,"specialCharacters":true}"#);
        let value: serde_json::Value = serde_json::from_str(&response).expect("json response");

        assert_eq!(value["ok"], true);
        assert_eq!(value["data"]["password"].as_str().unwrap().len(), 16);
    }

    #[test]
    fn unknown_request_returns_invalid_request_error() {
        let response = handle_request_json(r#"{"type":"missing"}"#);
        let value: serde_json::Value = serde_json::from_str(&response).expect("json response");

        assert_eq!(value["ok"], false);
        assert_eq!(value["error"]["code"], "invalid_request");
    }

    #[test]
    fn quit_request_returns_locked() {
        let response = handle_request_json(r#"{"type":"quit"}"#);
        let value: serde_json::Value = serde_json::from_str(&response).expect("json response");

        assert_eq!(value["ok"], true);
        assert_eq!(value["data"]["locked"], true);
    }
}
