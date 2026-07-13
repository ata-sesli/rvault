# Changelog

## 1.4.0

- Added typed `Ciphertext`, `SecretKey`, and `SecretBytes` crypto operations with `CryptoError`.
- Added `EntryRepository` and purpose-specific entry types with `StorageError`.
- Added `SessionKey::load` with `SessionError` while retaining the 1.x session adapter.
- Deprecated unsafe, stringly, clipboard-coupled, and no-op compatibility APIs.
- Migrated the CLI, TUI, and native host to typed core APIs without changing persisted formats.
- Published the [2.0 migration guide](docs/migration/rvault-core-2.0.md).

## RVault Core 2.0 breaking-change checklist

These items remain unchecked until their 2.0 implementation and verification land.

- [ ] Remove plaintext `Table::add_entry`; callers supply a `SecretKey` to `EntryRepository::add`.
- [ ] Remove error-swallowing `Table::add_entry_with_key` and `Table::remove_entry` wrappers.
- [ ] Remove clipboard-coupled `Table::get_password` and `Table::get_password_with_key`.
- [ ] Remove result-suffix compatibility methods after equivalent repository operations land,
      excluding `import_entry_with_key_result` until its separate replacement gate is satisfied.
- [ ] Remove `import_entry_with_key_result` only after a tested typed import boundary preserves
      timestamps, pin state, and upsert behavior.
- [ ] Replace `VaultEntry` with purpose-specific entry types.
- [ ] Remove placeholder `Vault` fields and no-op export/encryption methods.
- [ ] Remove unused `EncryptedData`, `DerivedEncryptedData`, `HashedData`, `Encryption`, and `Hash` types.
- [ ] Remove `encrypt_data`.
- [ ] Replace Base64/string crypto helpers with typed crypto operations.
- [ ] Replace raw secret-key returns with opaque zeroizing types.
- [ ] Replace public string errors with non-exhaustive typed errors.
- [ ] Stop mapping crypto failures to `rusqlite::Error::InvalidQuery`.
- [ ] Remove printing, clipboard access, and user-facing messaging from `rvault-core`.
- [ ] Make SQLite connections, persisted rows, codecs, and migrations private.
- [ ] Return typed errors for malformed values, corrupt data, and unsupported versions.
- [ ] Return `StorageError::NotFound` consistently for missing update, pin, and removal targets.
- [ ] Return `StorageError::Conflict` for duplicate entry identities.
- [ ] Remove unused public dependencies and unfinished keystore scaffolding.
