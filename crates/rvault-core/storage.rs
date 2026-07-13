use crate::{clipboard::copy_text, error::DatabaseError, secret::SecretKey, vault::VaultEntry};
use chrono::Utc;
use directories::ProjectDirs;
use rusqlite::{Connection, params};
use std::path::PathBuf;

mod error;
mod migration;
mod repository;

pub use error::StorageError;
pub use repository::{
    DecryptedEntry, EntryMetadata, EntryRepository, EntrySelector, EntryUpdate, NewEntry,
};

const CURRENT_DB_PATH: &str = "RVAULT_CURRENT_DB_PATH";
const CURRENT_VAULT_NAME: &str = "RVAULT_CURRENT_VAULT_NAME";

pub struct Database {
    connection: Connection,
}
impl Database {
    pub fn new() -> Result<Self, DatabaseError> {
        let final_path = database_path()?;
        let connection = Connection::open(&final_path)?;
        Ok(Self { connection })
    }
}

pub fn database_path() -> Result<PathBuf, DatabaseError> {
    if let Some(project_dirs) = ProjectDirs::from("io.github", "ata-sesli", "RVault") {
        let project_dirs = project_dirs.data_dir();
        let database_dir = project_dirs.join("databases");
        let _ = std::fs::create_dir_all(&database_dir)?;
        Ok(database_dir.join("default_vault.sqlite"))
    } else {
        Err(DatabaseError::Path)
    }
}

pub struct Table {
    table_name: String,
}
impl Table {
    pub fn new(db: &Database, table_name: Option<String>) -> Result<Self, DatabaseError> {
        let connection = &db.connection;
        let full_table_name = match table_name {
            Some(name) => {
                if Self::is_valid_identifier(&name) {
                    name
                } else {
                    return Err(DatabaseError::Sqlite(
                        rusqlite::Error::InvalidParameterName(name),
                    ));
                }
            }
            None => String::from("main"),
        };
        let query = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                platform TEXT NOT NULL,
                user_id TEXT NOT NULL,
                password TEXT NOT NULL,
                nonce TEXT,
                salt TEXT,
                pinned BOOLEAN DEFAULT FALSE,
                created_at INTEGER DEFAULT 0,
                updated_at INTEGER DEFAULT 0,
                UNIQUE(platform, user_id)
                )",
            full_table_name
        );
        connection.execute(&query, [])?;
        migration::migrate(connection, &full_table_name)?;
        Ok(Self {
            table_name: full_table_name,
        })
    }
    #[deprecated(
        note = "no equally safe replacement exists; use EntryRepository::add with SecretKey"
    )]
    pub fn add_entry(&self, db: &Database, platform: String, id_and_password: String) {
        let (user_id, password) = id_and_password.split_once(':').unwrap();
        let query = format!(
            "INSERT INTO {} (platform,user_id,password)
             VALUES (?1,?2,?3)
            ",
            &self.table_name
        );
        let _ = db.connection.execute(
            &query,
            [
                platform.to_string(),
                user_id.to_string(),
                password.to_string(),
            ],
        );
    }
    #[deprecated(note = "use EntryRepository::remove")]
    pub fn remove_entry(&self, db: &Database, platform: String, user_id: String) {
        let _ = self.remove_entry_impl(db, platform, user_id);
    }
    #[deprecated(note = "use EntryRepository::get and copy only at the application boundary")]
    pub fn get_password(
        &self,
        db: &Database,
        platform: String,
        user_id: String,
    ) -> Result<(), DatabaseError> {
        // Legacy/plaintext path: keep behavior for existing rows
        let query: String = format!(
            "SELECT password FROM {}
             WHERE platform = (?1) AND user_id = (?2)",
            &self.table_name
        );
        let password_result =
            db.connection
                .query_row(&query, [platform.to_string(), user_id.to_string()], |row| {
                    row.get::<_, String>(0)
                });
        match password_result {
            Ok(password) => {
                let _ = copy_text(password);
                Ok(())
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                eprintln!(
                    "Error: No entry found for user '{}' on platform '{}'.",
                    user_id, platform
                );
                Err(DatabaseError::from(rusqlite::Error::QueryReturnedNoRows))
            }
            Err(e) => {
                eprintln!("Database query failed: {e}");
                Err(DatabaseError::from(e))
            }
        }
    }
    /// Adds an entry using the main Encryption Key to derive a unique key for this entry.
    #[deprecated(note = "use EntryRepository::add")]
    pub fn add_entry_with_key(
        &self,
        db: &Database,
        encryption_key: &[u8],
        platform: String,
        id_and_password: String,
    ) {
        let (user_id, password) = id_and_password.split_once(':').unwrap();
        let _ = self.add_entry_with_key_impl(
            db,
            encryption_key,
            platform,
            user_id.to_string(),
            password.to_string(),
        );
    }

    #[deprecated(note = "use EntryRepository::add, or EntryRepository::update for replacement")]
    pub fn add_entry_with_key_result(
        &self,
        db: &Database,
        encryption_key: &[u8],
        platform: String,
        user_id: String,
        password: String,
    ) -> Result<(), DatabaseError> {
        self.add_entry_with_key_impl(db, encryption_key, platform, user_id, password)
    }

    fn add_entry_with_key_impl(
        &self,
        db: &Database,
        encryption_key: &[u8],
        platform: String,
        user_id: String,
        password: String,
    ) -> Result<(), DatabaseError> {
        let key = secret_key_from_slice(encryption_key)?;
        let (ciphertext, nonce, salt) =
            repository::encrypt_entry(&key, password.as_bytes()).map_err(map_storage_error)?;

        let now = Utc::now().timestamp();

        let query = format!(
            "INSERT INTO {} (platform, user_id, password, nonce, salt, pinned, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, FALSE, ?6, ?7)
             ON CONFLICT(platform, user_id) DO UPDATE SET
             password = ?3,
             nonce = ?4,
             salt = ?5,
             updated_at = ?7;",
             &self.table_name
        );
        db.connection.execute(
            &query,
            params![platform, user_id, ciphertext, nonce, salt, now, now],
        )?;
        Ok(())
    }

    #[deprecated(note = "use EntryRepository::remove")]
    pub fn remove_entry_result(
        &self,
        db: &Database,
        platform: String,
        user_id: String,
    ) -> Result<(), DatabaseError> {
        self.remove_entry_impl(db, platform, user_id)
    }

    fn remove_entry_impl(
        &self,
        db: &Database,
        platform: String,
        user_id: String,
    ) -> Result<(), DatabaseError> {
        let query = format!(
            "DELETE FROM {}
             WHERE platform = (?1) AND user_id = (?2)
            ",
            &self.table_name
        );
        let affected = db
            .connection
            .execute(&query, [platform.to_string(), user_id.to_string()])?;
        if affected == 0 {
            return Err(DatabaseError::Sqlite(rusqlite::Error::QueryReturnedNoRows));
        }
        Ok(())
    }

    #[deprecated(
        note = "use EntryRepository::list_metadata for application-level existence checks"
    )]
    pub fn entry_exists(
        &self,
        db: &Database,
        platform: &str,
        user_id: &str,
    ) -> Result<bool, DatabaseError> {
        self.entry_exists_impl(db, platform, user_id)
    }

    fn entry_exists_impl(
        &self,
        db: &Database,
        platform: &str,
        user_id: &str,
    ) -> Result<bool, DatabaseError> {
        let query = format!(
            "SELECT EXISTS(SELECT 1 FROM {} WHERE platform = ?1 AND user_id = ?2)",
            &self.table_name
        );
        let exists: bool = db
            .connection
            .query_row(&query, params![platform, user_id], |row| row.get(0))?;
        Ok(exists)
    }

    #[deprecated(
        note = "no typed replacement preserves imported timestamps and pin state; isolate this import-only adapter before 2.0"
    )]
    pub fn import_entry_with_key_result(
        &self,
        db: &Database,
        encryption_key: &[u8],
        entry: &crate::portable_export::ExportEntry,
    ) -> Result<(), DatabaseError> {
        self.import_entry_with_key_impl(db, encryption_key, entry)
    }

    fn import_entry_with_key_impl(
        &self,
        db: &Database,
        encryption_key: &[u8],
        entry: &crate::portable_export::ExportEntry,
    ) -> Result<(), DatabaseError> {
        let key = secret_key_from_slice(encryption_key)?;
        let (ciphertext, nonce, salt) = repository::encrypt_entry(&key, entry.password.as_bytes())
            .map_err(map_storage_error)?;
        let now = Utc::now().timestamp();
        let created_at = if entry.created_at > 0 {
            entry.created_at
        } else {
            now
        };
        let updated_at = if entry.updated_at > 0 {
            entry.updated_at
        } else {
            now
        };
        let query = format!(
            "INSERT INTO {} (platform, user_id, password, nonce, salt, pinned, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(platform, user_id) DO UPDATE SET
             password = ?3,
             nonce = ?4,
             salt = ?5,
             pinned = ?6,
             updated_at = ?8;",
            &self.table_name
        );
        db.connection.execute(
            &query,
            params![
                entry.platform,
                entry.user_id,
                ciphertext,
                nonce,
                salt,
                entry.pinned,
                created_at,
                updated_at
            ],
        )?;
        Ok(())
    }

    /// Updates an existing entry's User ID and/or Password.
    /// Platform is used as a lookup key along with the OLD User ID, and cannot be changed.
    #[deprecated(note = "use EntryRepository::update")]
    pub fn update_entry(
        &self,
        db: &Database,
        encryption_key: &[u8],
        platform: &str,
        old_user_id: &str,
        new_user_id: &str,
        new_password: &str,
    ) -> Result<(), DatabaseError> {
        self.update_entry_impl(
            db,
            encryption_key,
            platform,
            old_user_id,
            new_user_id,
            new_password,
        )
    }

    fn update_entry_impl(
        &self,
        db: &Database,
        encryption_key: &[u8],
        platform: &str,
        old_user_id: &str,
        new_user_id: &str,
        new_password: &str,
    ) -> Result<(), DatabaseError> {
        let key = secret_key_from_slice(encryption_key)?;
        let (ciphertext, nonce, salt) =
            repository::encrypt_entry(&key, new_password.as_bytes()).map_err(map_storage_error)?;
        let now = Utc::now().timestamp();

        // 3. Update the entry
        // We must ensure that if we are changing the user_id, the new user_id doesn't already exist for this platform
        if old_user_id != new_user_id {
            let check_query = format!(
                "SELECT COUNT(*) FROM {} WHERE platform = ?1 AND user_id = ?2",
                &self.table_name
            );
            let count: i64 =
                db.connection
                    .query_row(&check_query, [platform, new_user_id], |r| r.get(0))?;
            if count > 0 {
                return Err(DatabaseError::Sqlite(rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(19), // Constraint violation code roughly
                    Some("User ID already exists for this platform".into()),
                )));
            }
        }

        let query = format!(
            "UPDATE {} SET user_id = ?1, password = ?2, nonce = ?3, salt = ?4, updated_at = ?5 WHERE platform = ?6 AND user_id = ?7",
            &self.table_name
        );

        let affected = db.connection.execute(
            &query,
            params![
                new_user_id,
                ciphertext,
                nonce,
                salt,
                now,
                platform,
                old_user_id
            ],
        )?;
        if affected != 1 {
            return Err(DatabaseError::Sqlite(rusqlite::Error::QueryReturnedNoRows));
        }

        Ok(())
    }

    #[deprecated(note = "use EntryRepository::set_pinned")]
    pub fn toggle_pin(
        &self,
        db: &Database,
        platform: String,
        user_id: String,
    ) -> Result<bool, DatabaseError> {
        self.toggle_pin_impl(db, platform, user_id)
    }

    fn toggle_pin_impl(
        &self,
        db: &Database,
        platform: String,
        user_id: String,
    ) -> Result<bool, DatabaseError> {
        // Check current state
        let query_check = format!(
            "SELECT pinned FROM {} WHERE platform = ?1 AND user_id = ?2",
            &self.table_name
        );
        let current_pinned: bool =
            db.connection
                .query_row(&query_check, [&platform, &user_id], |row| row.get(0))?;

        if !current_pinned {
            // Check cap
            let query_count = format!(
                "SELECT COUNT(*) FROM {} WHERE pinned = TRUE",
                &self.table_name
            );
            let count: i64 = db
                .connection
                .query_row(&query_count, [], |row| row.get(0))?;
            if count >= 10 {
                return Err(DatabaseError::Sqlite(rusqlite::Error::InvalidQuery)); // Or custom error "Pin limit reached"
            }
        }

        let new_state = !current_pinned;
        let query_update = format!(
            "UPDATE {} SET pinned = ?1 WHERE platform = ?2 AND user_id = ?3",
            &self.table_name
        );
        let affected = db
            .connection
            .execute(&query_update, params![new_state, platform, user_id])?;
        if affected != 1 {
            return Err(DatabaseError::Sqlite(rusqlite::Error::QueryReturnedNoRows));
        }
        Ok(new_state)
    }

    /// Retrieves the decrypted password for an entry.
    /// Returns the plaintext password if successful.
    #[deprecated(note = "use EntryRepository::get")]
    pub fn retrieve_password_with_key(
        &self,
        db: &Database,
        encryption_key: &[u8],
        platform: String,
        user_id: String,
    ) -> Result<String, DatabaseError> {
        self.retrieve_password_with_key_impl(db, encryption_key, platform, user_id)
    }

    fn retrieve_password_with_key_impl(
        &self,
        db: &Database,
        encryption_key: &[u8],
        platform: String,
        user_id: String,
    ) -> Result<String, DatabaseError> {
        let query = format!(
            "SELECT password, nonce, salt FROM {} WHERE platform = (?1) AND user_id = (?2)",
            &self.table_name
        );

        let row =
            db.connection
                .query_row(&query, [platform.to_string(), user_id.to_string()], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                });

        match row {
            Ok((ciphertext, nonce, salt_str)) => {
                let key = secret_key_from_slice(encryption_key)?;
                let plaintext = repository::decrypt_entry(&key, &ciphertext, &nonce, &salt_str)
                    .map_err(map_storage_error)?;
                std::str::from_utf8(plaintext.expose())
                    .map(str::to_owned)
                    .map_err(|error| DatabaseError::Crypto(error.to_string()))
            }
            Err(e) => {
                eprintln!("Database query failed: {}", e);
                Err(DatabaseError::from(e))
            }
        }
    }

    /// Retrieves an entry by re-deriving its unique key from the main Encryption Key and the entry's salt.
    /// Copies the password to clipboard and prints success message.
    #[deprecated(note = "use EntryRepository::get and copy only at the application boundary")]
    pub fn get_password_with_key(
        &self,
        db: &Database,
        encryption_key: &[u8],
        platform: String,
        user_id: String,
    ) -> Result<(), DatabaseError> {
        match self.retrieve_password_with_key_impl(db, encryption_key, platform, user_id) {
            Ok(plaintext) => {
                copy_text(plaintext);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
    #[deprecated(note = "use EntryRepository::list_metadata")]
    pub fn list(&self, db: &Database) -> Result<Vec<VaultEntry>, DatabaseError> {
        self.list_impl(db)
    }

    fn list_impl(&self, db: &Database) -> Result<Vec<VaultEntry>, DatabaseError> {
        let query = format!(
            "SELECT id, platform, user_id, password, salt, nonce, pinned, created_at, updated_at FROM {} ORDER BY pinned DESC, platform ASC",
            &self.table_name
        );
        let mut statement = db.connection.prepare(&query)?;
        let rows = statement.query_map([], |row| {
            Ok(VaultEntry {
                id: row.get(0)?,
                platform: row.get(1)?,
                user_id: row.get(2)?,
                password: row.get(3)?,
                salt: row.get(4)?,
                nonce: row.get(5)?,
                pinned: row.get(6)?,
                created_at: row.get(7).unwrap_or(0),
                updated_at: row.get(8).unwrap_or(0),
            })
        })?;

        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }
    fn is_valid_identifier(name: &str) -> bool {
        !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_')
    }
}

fn secret_key_from_slice(key: &[u8]) -> Result<SecretKey, DatabaseError> {
    let bytes: [u8; 32] = key
        .try_into()
        .map_err(|_| DatabaseError::Crypto("invalid key length".to_string()))?;
    Ok(SecretKey::from_bytes(bytes))
}

fn map_storage_error(error: StorageError) -> DatabaseError {
    match error {
        StorageError::NotFound => DatabaseError::Sqlite(rusqlite::Error::QueryReturnedNoRows),
        StorageError::Conflict => DatabaseError::Sqlite(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(19),
            Some("entry conflict".to_string()),
        )),
        StorageError::Schema(error) | StorageError::Database(error) => DatabaseError::Sqlite(error),
        StorageError::Io(error) => DatabaseError::Io(error),
        StorageError::Crypto(error) => DatabaseError::Crypto(error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn memory_db() -> Database {
        Database {
            connection: Connection::open_in_memory().expect("in-memory database"),
        }
    }

    #[test]
    fn valid_custom_vault_name_is_accepted() {
        let db = memory_db();

        let table = Table::new(&db, Some("work_accounts".to_string()));

        assert!(table.is_ok());
    }

    #[test]
    fn invalid_custom_vault_name_is_rejected() {
        let db = memory_db();

        let table = Table::new(&db, Some("bad-name".to_string()));

        assert!(table.is_err());
    }

    #[test]
    fn result_crud_helpers_round_trip_encrypted_entry() {
        let db = memory_db();
        let table = Table::new(&db, None).expect("main table");
        let key = [7_u8; 32];

        table
            .add_entry_with_key_impl(
                &db,
                &key,
                "github".to_string(),
                "alice".to_string(),
                "old-pass".to_string(),
            )
            .expect("create entry");

        let created = table
            .retrieve_password_with_key_impl(&db, &key, "github".to_string(), "alice".to_string())
            .expect("retrieve created password");
        assert_eq!(created, "old-pass");

        table
            .update_entry_impl(
                &db,
                &key,
                "github",
                "alice",
                "alice@example.com",
                "new-pass",
            )
            .expect("update entry");

        let updated = table
            .retrieve_password_with_key_impl(
                &db,
                &key,
                "github".to_string(),
                "alice@example.com".to_string(),
            )
            .expect("retrieve updated password");
        assert_eq!(updated, "new-pass");

        table
            .remove_entry_impl(&db, "github".to_string(), "alice@example.com".to_string())
            .expect("remove entry");

        let deleted = table.retrieve_password_with_key_impl(
            &db,
            &key,
            "github".to_string(),
            "alice@example.com".to_string(),
        );
        assert!(deleted.is_err());
    }

    fn assert_missing_row(error: DatabaseError) {
        assert!(matches!(
            error,
            DatabaseError::Sqlite(rusqlite::Error::QueryReturnedNoRows)
        ));
    }

    #[test]
    fn missing_entry_toggle_pin_returns_no_rows() {
        let db = memory_db();
        let table = Table::new(&db, None).unwrap();
        assert_missing_row(
            table
                .toggle_pin_impl(&db, "missing".to_string(), "user".to_string())
                .unwrap_err(),
        );
    }

    #[test]
    fn missing_entry_update_returns_no_rows() {
        let db = memory_db();
        let table = Table::new(&db, None).unwrap();
        assert_missing_row(
            table
                .update_entry_impl(&db, &[7_u8; 32], "missing", "user", "new-user", "secret")
                .unwrap_err(),
        );
    }

    #[test]
    fn missing_entry_remove_returns_no_rows() {
        let db = memory_db();
        let table = Table::new(&db, None).unwrap();
        assert_missing_row(
            table
                .remove_entry_impl(&db, "missing".to_string(), "user".to_string())
                .unwrap_err(),
        );
    }

    #[test]
    fn toggle_pin_propagates_query_failure() {
        let db = memory_db();
        let table = Table::new(&db, None).unwrap();
        db.connection.execute("DROP TABLE main", []).unwrap();
        assert!(matches!(
            table.toggle_pin_impl(&db, "missing".to_string(), "user".to_string()),
            Err(DatabaseError::Sqlite(_))
        ));
    }

    #[test]
    fn typed_api_classifies_missing_storage_row() {
        let db = memory_db();
        let repository = EntryRepository::new(&db, None).unwrap();
        let key = crate::secret::SecretKey::from_bytes([7_u8; 32]);

        let error = match repository.get(&key, EntrySelector::new("missing", "user")) {
            Err(error) => error,
            Ok(_) => panic!("missing entry was returned"),
        };

        assert!(matches!(error, StorageError::NotFound));
    }

    #[test]
    fn typed_api_classifies_duplicate_identity_conflict() {
        let db = memory_db();
        let repository = EntryRepository::new(&db, None).unwrap();
        let key = crate::secret::SecretKey::from_bytes([7_u8; 32]);
        repository
            .add(&key, NewEntry::new("github", "alice", b"first"))
            .unwrap();

        let error = repository
            .add(&key, NewEntry::new("github", "alice", b"second"))
            .unwrap_err();

        assert!(matches!(error, StorageError::Conflict));
    }

    fn repository_with_ten_pins<'a>(db: &'a Database, key: &SecretKey) -> EntryRepository<'a> {
        let repository = EntryRepository::new(db, None).unwrap();
        for index in 0..10 {
            let user_id = format!("user-{index}");
            repository
                .add(key, NewEntry::new("pins", &user_id, b"secret"))
                .unwrap();
            repository
                .set_pinned(EntrySelector::new("pins", &user_id), true)
                .unwrap();
        }
        repository
    }

    #[test]
    fn typed_pin_missing_target_is_not_found_even_at_cap() {
        let db = memory_db();
        let key = crate::secret::SecretKey::from_bytes([7_u8; 32]);
        let repository = repository_with_ten_pins(&db, &key);

        assert!(matches!(
            repository.set_pinned(EntrySelector::new("missing", "user"), true),
            Err(StorageError::NotFound)
        ));
    }

    #[test]
    fn typed_pin_is_idempotent_for_an_already_pinned_target_at_cap() {
        let db = memory_db();
        let key = crate::secret::SecretKey::from_bytes([7_u8; 32]);
        let repository = repository_with_ten_pins(&db, &key);

        repository
            .set_pinned(EntrySelector::new("pins", "user-0"), true)
            .unwrap();
        assert!(
            repository
                .get(&key, EntrySelector::new("pins", "user-0"))
                .unwrap()
                .metadata
                .pinned
        );
    }

    #[test]
    fn typed_api_repository_crud_returns_secrets_and_metadata() {
        let db = memory_db();
        let repository = EntryRepository::new(&db, None).unwrap();
        let key = crate::secret::SecretKey::from_bytes([7_u8; 32]);
        repository
            .add(&key, NewEntry::new("github", "alice", b"old-pass"))
            .unwrap();

        let created = repository
            .get(&key, EntrySelector::new("github", "alice"))
            .unwrap();
        assert_eq!(created.secret.expose(), b"old-pass");
        assert_eq!(created.metadata.platform, "github");
        assert_eq!(repository.list_metadata().unwrap().len(), 1);

        repository
            .update(
                &key,
                EntrySelector::new("github", "alice"),
                EntryUpdate::new("alice@example.com", b"new-pass"),
            )
            .unwrap();
        repository
            .set_pinned(EntrySelector::new("github", "alice@example.com"), true)
            .unwrap();
        let updated = repository
            .get(&key, EntrySelector::new("github", "alice@example.com"))
            .unwrap();
        assert_eq!(updated.secret.expose(), b"new-pass");
        assert!(updated.metadata.pinned);

        repository
            .remove(EntrySelector::new("github", "alice@example.com"))
            .unwrap();
        assert!(matches!(
            repository.get(&key, EntrySelector::new("github", "alice@example.com")),
            Err(StorageError::NotFound)
        ));
    }
}
