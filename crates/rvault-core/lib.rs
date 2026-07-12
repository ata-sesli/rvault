//! Shared configuration, cryptography, storage, session, backup, and vault logic for RVault.
//!
//! The public APIs in [`crypto`], [`session`], and [`storage`] include legacy surfaces that remain
//! stable for the rest of the 1.x release line. New applications should prefer result-returning
//! operations where both legacy and fallible variants are available.

pub mod config;
pub mod error;

pub mod backup;
pub mod binary;

pub mod crypto;
pub mod identity;
pub mod keystore;

pub mod portable_export;
pub mod secret;
pub mod session;
pub mod storage;
pub mod vault;

pub mod clipboard;
pub mod watcher;

pub use error::{ConfigError, DatabaseError};
pub use vault::VaultEntry;

/// Locks the vault by ending the current session.
/// This does NOT clear the clipboard contents, per user preference.
pub fn lock() -> Result<(), String> {
    session::end_session()
}
