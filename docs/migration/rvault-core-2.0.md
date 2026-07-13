# Migrating `rvault-core` from 1.x to 2.0

This guide describes the planned 2.0 public API. RVault 1.4 provides the typed replacement APIs
alongside deprecated compatibility APIs so applications can migrate before upgrading to 2.0.

## Supported versions

Upgrade from any supported 1.x release to 1.4 first. The 1.4 release reads the existing SQLite,
keystore, session, backup, portable-export, and identity formats and exposes both API generations.
The compatibility APIs are removed in 2.0.

## Upgrade order

1. Upgrade dependencies and applications to 1.4 without changing stored data.
2. Replace deprecated calls and resolve every deprecation warning.
3. Keep clipboard, printing, and other presentation behavior in the application.
4. Run application tests against existing vault fixtures.
5. Upgrade to 2.0 only after the application builds without deprecated APIs.

## Removed APIs

### `Table::add_entry`

The plaintext, error-swallowing API has no contract-preserving replacement. Callers must load or
derive a `SecretKey`, keep the password separate from its identity, and handle storage failure.

```rust
// 1.x
table.add_entry(&db, "example.com".into(), "me:password".into());

// 2.0
use rvault_core::{EntryRepository, NewEntry, SecretKey, StorageError};

fn add(db: &rvault_core::storage::Database, key: &SecretKey) -> Result<(), StorageError> {
    EntryRepository::new(db, None)?.add(
        key,
        NewEntry::new("example.com", "me", b"password"),
    )
}
```

### `Table::add_entry_with_key`

```rust
// 1.x: accepted raw key bytes and a combined identity/secret string, and returned ().
table.add_entry_with_key(
    &db,
    &key_bytes,
    "example.com".into(),
    "me:password".into(),
);

// 2.0: the key is opaque and duplicate identity is StorageError::Conflict.
let key = rvault_core::SecretKey::from_bytes(key_bytes);
let repo = rvault_core::EntryRepository::new(&db, None)?;
repo.add(&key, rvault_core::NewEntry::new("example.com", "me", b"password"))?;
```

### `Table::add_entry_with_key_result`

```rust
// 1.x: separate identity/secret strings and DatabaseError.
table.add_entry_with_key_result(
    &db,
    &key_bytes,
    "example.com".into(),
    "me".into(),
    "password".into(),
)?;

// 2.0: explicit secret ownership and StorageError.
let key = rvault_core::SecretKey::from_bytes(key_bytes);
let repo = rvault_core::EntryRepository::new(&db, None)?;
repo.add(&key, rvault_core::NewEntry::new("example.com", "me", b"password"))?;
```

### `Table::remove_entry` and result-suffix removal helpers

```rust
// 1.x: the wrapper discarded the result.
table.remove_entry(&db, "example.com".into(), "me".into());

// 2.0: absence is StorageError::NotFound.
let repo = rvault_core::EntryRepository::new(&db, None)?;
repo.remove(rvault_core::EntrySelector::new("example.com", "me"))?;
```

The `*_result` compatibility names are also removed. Use the corresponding repository operation:
`add`, `update`, `remove`, `get`, `list_metadata`, or `set_pinned`.

### `Table::remove_entry_result`

```rust
// 1.x
table.remove_entry_result(&db, "example.com".into(), "me".into())?;

// 2.0
let repo = rvault_core::EntryRepository::new(&db, None)?;
repo.remove(rvault_core::EntrySelector::new("example.com", "me"))?;
```

The new operation returns `StorageError::NotFound` for a missing identity instead of wrapping
`rusqlite::Error::QueryReturnedNoRows` in `DatabaseError`.

### `Table::import_entry_with_key_result`

There is no typed public import operation in 1.4 that preserves imported timestamps and pin state.
During the 1.4 migration, isolate the deprecated call at the import boundary rather than replacing
it with ordinary `add`/`update` calls that would silently change metadata behavior:

```rust
// 1.x: called throughout import orchestration.
table.import_entry_with_key_result(&db, &key_bytes, &export_entry)?;

// 1.4 preparation: one temporary, explicit compatibility boundary.
#[allow(deprecated)]
fn import_preserving_metadata(
    table: &rvault_core::storage::Table,
    db: &rvault_core::storage::Database,
    key: &[u8],
    entry: &rvault_core::portable_export::ExportEntry,
) -> Result<(), rvault_core::DatabaseError> {
    table.import_entry_with_key_result(db, key, entry)
}
```

The 2.0 import boundary must replace this isolated adapter before removal. This guide does not claim
that such a typed replacement exists in 1.4.

### `Table::entry_exists`

```rust
// 1.x
let exists = table.entry_exists(&db, "example.com", "me")?;

// 2.0: make the application-level decision from non-secret metadata.
let repo = rvault_core::EntryRepository::new(&db, None)?;
let exists = repo
    .list_metadata()?
    .iter()
    .any(|entry| entry.platform == "example.com" && entry.user_id == "me");
```

### `Table::update_entry`

```rust
// 1.x
table.update_entry(
    &db,
    &key_bytes,
    "example.com",
    "old-user",
    "new-user",
    "new-password",
)?;

// 2.0
let key = rvault_core::SecretKey::from_bytes(key_bytes);
let repo = rvault_core::EntryRepository::new(&db, None)?;
repo.update(
    &key,
    rvault_core::EntrySelector::new("example.com", "old-user"),
    rvault_core::EntryUpdate::new("new-user", b"new-password"),
)?;
```

### `Table::toggle_pin`

```rust
// 1.x: toggled implicitly and returned the new state.
let pinned = table.toggle_pin(&db, "example.com".into(), "me".into())?;

// 2.0: the application chooses the explicit target state.
let repo = rvault_core::EntryRepository::new(&db, None)?;
let current = repo
    .list_metadata()?
    .into_iter()
    .find(|entry| entry.platform == "example.com" && entry.user_id == "me")
    .ok_or(rvault_core::StorageError::NotFound)?;
let pinned = !current.pinned;
repo.set_pinned(rvault_core::EntrySelector::new("example.com", "me"), pinned)?;
```

### `Table::retrieve_password_with_key`

```rust
// 1.x: returned an ordinary String.
let password = table.retrieve_password_with_key(
    &db,
    &key_bytes,
    "example.com".into(),
    "me".into(),
)?;

// 2.0: decrypted bytes remain in zeroizing ownership.
let key = rvault_core::SecretKey::from_bytes(key_bytes);
let entry = rvault_core::EntryRepository::new(&db, None)?.get(
    &key,
    rvault_core::EntrySelector::new("example.com", "me"),
)?;
consume(entry.secret.expose());
```

### `Table::list`

```rust
// 1.x: exposed VaultEntry values containing persisted secret fields.
let entries: Vec<rvault_core::VaultEntry> = table.list(&db)?;

// 2.0: list views receive metadata only.
let repo = rvault_core::EntryRepository::new(&db, None)?;
let entries: Vec<rvault_core::EntryMetadata> = repo.list_metadata()?;
```

### `Table::get_password` and `Table::get_password_with_key`

```rust
// 1.x: core decrypted and copied to the clipboard.
table.get_password_with_key(&db, &key_bytes, "example.com".into(), "me".into())?;

// 2.0: core returns zeroizing bytes; the application chooses their destination.
let key = rvault_core::SecretKey::from_bytes(key_bytes);
let entry = rvault_core::EntryRepository::new(&db, None)?.get(
    &key,
    rvault_core::EntrySelector::new("example.com", "me"),
)?;
application_clipboard.write(entry.secret.expose())?;
```

### `encrypt_data` and `EncryptedData`

```rust
// 1.x: returned a printable key beside Base64 strings.
let encrypted: rvault_core::crypto::EncryptedData =
    rvault_core::crypto::encrypt_data(b"secret")?;

// 2.0: key ownership remains with the caller and ciphertext fields are read-only.
let key = rvault_core::SecretKey::from_bytes(key_bytes);
let encrypted: rvault_core::Ciphertext = rvault_core::encrypt(&key, b"secret")?;
persist(encrypted.nonce(), encrypted.bytes())?;
```

### `encrypt_with_key` and `decrypt_with_key`

```rust
// 1.x: raw key input, Base64 output, and String errors.
let (ciphertext_b64, nonce_b64) =
    rvault_core::crypto::encrypt_with_key(&key_bytes, b"secret")?;
let plaintext = rvault_core::crypto::decrypt_with_key(
    &key_bytes,
    &ciphertext_b64,
    &nonce_b64,
)?;

// 2.0: validated ciphertext and typed crypto errors.
let key = rvault_core::SecretKey::from_bytes(key_bytes);
let ciphertext = rvault_core::encrypt(&key, b"secret")?;
let plaintext: rvault_core::SecretBytes = rvault_core::decrypt(&key, &ciphertext)?;
consume(plaintext.expose());
```

After decoding persisted bytes with the application's codec, callers reconstruct a `Ciphertext`
with `Ciphertext::try_from_parts`; that constructor reports an invalid nonce length. Codec failures
must be mapped separately to `CryptoError::InvalidEncoding`.

### Empty `Vault` export methods

`Vault::export_vault` and `Vault::export_partial_vault` performed no work and are removed.

```rust
// 1.x: no output was produced.
rvault_core::vault::Vault::export_vault();

// 2.0: select/decrypt entries explicitly, then create recipient-bound bytes.
let entries = build_export_entries(&repo, &key, selectors)?;
let bytes = rvault_core::portable_export::create_export_bytes(&recipient_code, &entries)?;
write_export_file(bytes)?;
```

## Renamed and replaced APIs

`VaultEntry` is split by purpose: `NewEntry` for creation, `EntrySelector` for identity,
`EntryUpdate` for replacement values, `EntryMetadata` for non-secret listings, and
`DecryptedEntry` for retrieval. The unfinished `Vault` facade and its placeholder fields are not a
2.0 storage boundary; use `EntryRepository` and the concrete backup/export modules.

```rust
// 1.x
let entry = rvault_core::vault::VaultEntry {
    platform: "example.com".into(), user_id: "me".into(), password: "password".into(),
    salt: None, nonce: None, pinned: false, id: None, created_at: 0, updated_at: 0,
};

// 2.0
let entry = rvault_core::NewEntry::new("example.com", "me", b"password");
repo.add(&key, entry)?;
let visible: Vec<rvault_core::EntryMetadata> = repo.list_metadata()?;
```

`generate_password` remains in 1.x for compatibility and uses an empty string as its short-length
sentinel. In migrated code, use typed rejection:

```rust
// 1.x
let password = rvault_core::crypto::generate_password(3, true);
assert!(password.is_empty());

// 2.0
let error = rvault_core::try_generate_password(3, true).unwrap_err();
assert!(matches!(error, rvault_core::CryptoError::InvalidPasswordLength { .. }));
```

## Error-model changes

Public crypto, storage, and session operations return non-exhaustive `CryptoError`, `StorageError`,
and `SessionError` enums instead of strings or unrelated SQLite errors. Match the variants you can
act on and retain a wildcard arm for future compatible additions.

```rust
match repo.get(&key, selector) {
    Ok(entry) => use_entry(entry),
    Err(rvault_core::StorageError::NotFound) => show_missing(),
    Err(rvault_core::StorageError::Conflict) => show_conflict(),
    Err(error) => return Err(error.into()),
}
```

Malformed key/nonce lengths, authentication failure, invalid encoding, short generated-password
lengths, invalid session tokens, expired sessions, corrupt sessions, missing rows, and duplicate
identities now have domain-specific variants. Core code no longer maps crypto failure to
`rusqlite::Error::InvalidQuery`.

## Secret-type changes

Keys and decrypted values use `SecretKey` and `SecretBytes`. They zeroize on drop and deliberately
do not implement `Clone`, `Debug`, `Display`, or serialization. Convert raw key material once with
`SecretKey::from_bytes`; borrow it with `as_bytes` only at a required codec boundary. Borrow
decrypted bytes with `SecretBytes::expose` for the shortest possible scope.

```rust
// 1.x
let raw: Vec<u8> = rvault_core::session::get_key_from_session()?;

// 2.0
let key: rvault_core::SecretKey = rvault_core::SessionKey::load()?;
```

## Storage behavior changes

`EntryRepository` is the canonical boundary. Duplicate `(platform, user_id)` values return
`StorageError::Conflict`. Missing update, pin, and removal targets return `StorageError::NotFound`.
Metadata never contains password, ciphertext, nonce, or salt fields. Direct connections, persisted
row types, codecs, and migrations are implementation details in 2.0.

## Session behavior changes

Use `SessionKey::load` rather than `get_key_from_session`. It validates the current token and maps
locked, expired, invalid-token, corrupt-session, and I/O conditions to `SessionError`. Existing
session files remain readable; the change is API ownership and error classification, not format.

## Clipboard separation

`rvault-core` no longer retrieves a password and copies it as one operation. Applications receive
a `DecryptedEntry`, decide whether to copy, display, or export it, and keep the exposed byte slice
short-lived. Core does not print user-facing status messages.

## Data-format compatibility

The 2.0 API migration does not intentionally rewrite data. The compatibility commitments are:

- Existing encrypted SQLite entries remain decryptable.
- Existing master-password hashes and keystores remain unlockable.
- Existing `.rvault-backup` files remain restorable.
- Existing `.rvault-export` files remain importable by their intended recipient.
- Existing identity files and public identity codes remain valid.

## CLI, TUI, and native-host impact

RVault 1.4 migrates its CLI, TUI, and browser native host to the typed repository, crypto, and
session APIs. Their command/protocol behavior stays stable. Clipboard writes remain in the CLI/TUI,
and native-host failures continue to map typed internal errors to stable protocol error codes.
Import paths retain their existing upsert, pinned, and timestamp behavior.

In 1.4, import code has a narrow `#[allow(deprecated)]` exception for `Table::entry_exists` and
`Table::import_entry_with_key_result`. There is not yet a typed public import API that preserves
timestamps and pin state; inventing one during the compatibility release would create a second
storage contract. Remove this exception when the 2.0 import boundary is implemented.

## Rollback instructions

Before upgrading the application, make an explicit user-chosen backup with the existing backup
command. If application code must roll back, restore the prior dependency/application version;
do not downgrade by rewriting the vault. Because 1.4 and 2.0 preserve the listed formats, the prior
version can reopen unchanged data. If a future security fix requires a format migration, follow
that release's dedicated migration and rollback instructions instead of this compatibility path.
