use rvault_core::{
    config::Config,
    crypto, session,
    storage::{Database, Table},
    vault::Vault,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

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

fn with_unlocked_table<F>(vault: Option<String>, operation: F) -> Result<Value, String>
where
    F: FnOnce(&Database, &Table, &[u8]) -> Result<Value, String>,
{
    let key = session::get_key_from_browser_session().map_err(|e| error("locked", e))?;
    let db = Database::new().map_err(|e| storage_error(e.to_string()))?;
    let table = Table::new(&db, vault).map_err(|e| storage_error(e.to_string()))?;
    operation(&db, &table, &key)
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
