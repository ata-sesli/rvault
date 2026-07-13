use argon2::{Argon2, password_hash::SaltString};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use chrono::Utc;
use rusqlite::{OptionalExtension, params};
use zeroize::Zeroizing;

use super::{Database, Table};
use crate::crypto::{Ciphertext, CryptoError, decrypt, encrypt};
use crate::secret::{SecretBytes, SecretKey};

use super::StorageError;

/// Borrowed values used to create an encrypted entry.
pub struct NewEntry<'a> {
    pub platform: &'a str,
    pub user_id: &'a str,
    pub secret: &'a [u8],
}

impl<'a> NewEntry<'a> {
    pub fn new(platform: &'a str, user_id: &'a str, secret: &'a [u8]) -> Self {
        Self {
            platform,
            user_id,
            secret,
        }
    }
}

/// Stable identity used to select one entry.
#[derive(Clone, Copy)]
pub struct EntrySelector<'a> {
    pub platform: &'a str,
    pub user_id: &'a str,
}

impl<'a> EntrySelector<'a> {
    pub fn new(platform: &'a str, user_id: &'a str) -> Self {
        Self { platform, user_id }
    }
}

/// Borrowed replacement values for an existing entry.
pub struct EntryUpdate<'a> {
    pub user_id: &'a str,
    pub secret: &'a [u8],
}

impl<'a> EntryUpdate<'a> {
    pub fn new(user_id: &'a str, secret: &'a [u8]) -> Self {
        Self { user_id, secret }
    }
}

/// Non-secret entry fields safe for list views.
pub struct EntryMetadata {
    pub id: Option<i64>,
    pub platform: String,
    pub user_id: String,
    pub pinned: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// One decrypted entry with zeroizing secret ownership.
pub struct DecryptedEntry {
    pub metadata: EntryMetadata,
    pub secret: SecretBytes,
}

/// Canonical encrypted-entry storage boundary.
pub struct EntryRepository<'a> {
    db: &'a Database,
    table: Table,
}

impl<'a> EntryRepository<'a> {
    pub fn new(db: &'a Database, table_name: Option<String>) -> Result<Self, StorageError> {
        let table = Table::new(db, table_name).map_err(map_database_error)?;
        Ok(Self { db, table })
    }

    pub fn add(&self, key: &SecretKey, entry: NewEntry<'_>) -> Result<(), StorageError> {
        let (ciphertext, nonce, salt) = encrypt_entry(key, entry.secret)?;
        let now = Utc::now().timestamp();
        let query = format!(
            "INSERT INTO {} (platform, user_id, password, nonce, salt, pinned, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, FALSE, ?6, ?7)",
            self.table.table_name
        );
        self.db
            .connection
            .execute(
                &query,
                params![
                    entry.platform,
                    entry.user_id,
                    ciphertext,
                    nonce,
                    salt,
                    now,
                    now
                ],
            )
            .map_err(map_insert_error)?;
        Ok(())
    }

    pub fn update(
        &self,
        key: &SecretKey,
        selector: EntrySelector<'_>,
        update: EntryUpdate<'_>,
    ) -> Result<(), StorageError> {
        if selector.user_id != update.user_id {
            let query = format!(
                "SELECT 1 FROM {} WHERE platform = ?1 AND user_id = ?2",
                self.table.table_name
            );
            if self
                .db
                .connection
                .query_row(&query, [selector.platform, update.user_id], |_| Ok(()))
                .optional()?
                .is_some()
            {
                return Err(StorageError::Conflict);
            }
        }
        let (ciphertext, nonce, salt) = encrypt_entry(key, update.secret)?;
        let query = format!(
            "UPDATE {} SET user_id = ?1, password = ?2, nonce = ?3, salt = ?4, updated_at = ?5 WHERE platform = ?6 AND user_id = ?7",
            self.table.table_name
        );
        let affected = self.db.connection.execute(
            &query,
            params![
                update.user_id,
                ciphertext,
                nonce,
                salt,
                Utc::now().timestamp(),
                selector.platform,
                selector.user_id
            ],
        )?;
        exactly_one(affected)
    }

    pub fn remove(&self, selector: EntrySelector<'_>) -> Result<(), StorageError> {
        let query = format!(
            "DELETE FROM {} WHERE platform = ?1 AND user_id = ?2",
            self.table.table_name
        );
        exactly_one(
            self.db
                .connection
                .execute(&query, [selector.platform, selector.user_id])?,
        )
    }

    pub fn get(
        &self,
        key: &SecretKey,
        selector: EntrySelector<'_>,
    ) -> Result<DecryptedEntry, StorageError> {
        let query = format!(
            "SELECT id, platform, user_id, password, nonce, salt, pinned, created_at, updated_at FROM {} WHERE platform = ?1 AND user_id = ?2",
            self.table.table_name
        );
        let (metadata, ciphertext, nonce, salt): (EntryMetadata, String, String, String) = self
            .db
            .connection
            .query_row(&query, [selector.platform, selector.user_id], |row| {
                Ok((
                    EntryMetadata {
                        id: row.get(0)?,
                        platform: row.get(1)?,
                        user_id: row.get(2)?,
                        pinned: row.get(6)?,
                        created_at: row.get(7).unwrap_or(0),
                        updated_at: row.get(8).unwrap_or(0),
                    },
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            })?;
        let entry_key = derive_entry_key(key, salt.as_bytes())?;
        let nonce = BASE64.decode(nonce).map_err(CryptoError::InvalidEncoding)?;
        let bytes = BASE64
            .decode(ciphertext)
            .map_err(CryptoError::InvalidEncoding)?;
        let ciphertext = Ciphertext::try_from_parts(&nonce, bytes)?;
        let secret = decrypt(&entry_key, &ciphertext)?;
        Ok(DecryptedEntry { metadata, secret })
    }

    pub fn list_metadata(&self) -> Result<Vec<EntryMetadata>, StorageError> {
        let query = format!(
            "SELECT id, platform, user_id, pinned, created_at, updated_at FROM {} ORDER BY pinned DESC, platform ASC",
            self.table.table_name
        );
        let mut statement = self.db.connection.prepare(&query)?;
        let rows = statement.query_map([], |row| {
            Ok(EntryMetadata {
                id: row.get(0)?,
                platform: row.get(1)?,
                user_id: row.get(2)?,
                pinned: row.get(3)?,
                created_at: row.get(4).unwrap_or(0),
                updated_at: row.get(5).unwrap_or(0),
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn set_pinned(
        &self,
        selector: EntrySelector<'_>,
        pinned: bool,
    ) -> Result<(), StorageError> {
        if pinned {
            let query = format!(
                "SELECT COUNT(*) FROM {} WHERE pinned = TRUE",
                self.table.table_name
            );
            let count: i64 = self.db.connection.query_row(&query, [], |row| row.get(0))?;
            if count >= 10 {
                return Err(StorageError::Conflict);
            }
        }
        let query = format!(
            "UPDATE {} SET pinned = ?1 WHERE platform = ?2 AND user_id = ?3",
            self.table.table_name
        );
        exactly_one(
            self.db
                .connection
                .execute(&query, params![pinned, selector.platform, selector.user_id])?,
        )
    }
}

fn derive_entry_key(key: &SecretKey, salt: &[u8]) -> Result<SecretKey, StorageError> {
    let mut bytes = Zeroizing::new([0_u8; 32]);
    Argon2::default()
        .hash_password_into(key.as_bytes(), salt, bytes.as_mut())
        .map_err(|error| CryptoError::KeyDerivation(error.to_string()))?;
    Ok(SecretKey::from_bytes(*bytes))
}

fn encrypt_entry(key: &SecretKey, secret: &[u8]) -> Result<(String, String, String), StorageError> {
    let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let entry_key = derive_entry_key(key, salt.as_ref().as_bytes())?;
    let ciphertext = encrypt(&entry_key, secret)?;
    Ok((
        BASE64.encode(ciphertext.bytes()),
        BASE64.encode(ciphertext.nonce()),
        salt.to_string(),
    ))
}

fn exactly_one(affected: usize) -> Result<(), StorageError> {
    if affected == 1 {
        Ok(())
    } else {
        Err(StorageError::NotFound)
    }
}

fn map_insert_error(error: rusqlite::Error) -> StorageError {
    match &error {
        rusqlite::Error::SqliteFailure(inner, _)
            if inner.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            StorageError::Conflict
        }
        _ => StorageError::Database(error),
    }
}

fn map_database_error(error: crate::error::DatabaseError) -> StorageError {
    match error {
        crate::error::DatabaseError::Path => StorageError::Schema(
            rusqlite::Error::InvalidParameterName("database path".to_string()),
        ),
        crate::error::DatabaseError::Io(error) => StorageError::Io(error),
        crate::error::DatabaseError::Sqlite(error) => StorageError::Schema(error),
        crate::error::DatabaseError::Crypto(error) => {
            StorageError::Crypto(CryptoError::KeyDerivation(error))
        }
    }
}
