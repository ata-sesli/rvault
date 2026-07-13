use thiserror::Error;

use crate::crypto::CryptoError;

/// Failures returned by [`super::EntryRepository`].
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum StorageError {
    #[error("entry not found")]
    NotFound,
    #[error("an entry with that identity already exists")]
    Conflict,
    #[error("storage schema error")]
    Schema(#[source] rusqlite::Error),
    #[error("database error")]
    Database(#[source] rusqlite::Error),
    #[error("storage I/O error")]
    Io(#[from] std::io::Error),
    #[error("cryptographic operation failed")]
    Crypto(#[from] CryptoError),
}

impl From<rusqlite::Error> for StorageError {
    fn from(error: rusqlite::Error) -> Self {
        match error {
            rusqlite::Error::QueryReturnedNoRows => Self::NotFound,
            error => Self::Database(error),
        }
    }
}
