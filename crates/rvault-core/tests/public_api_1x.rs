use rvault_core::{
    ConfigError, DatabaseError, VaultEntry,
    config::Config,
    crypto::{
        DerivedEncryptedData, EncryptedData, HashedData, decrypt_with_key, encrypt_data,
        encrypt_with_key, generate_password,
    },
    session::{get_key_from_session, start_session},
    storage::{Database, Table},
    vault::Vault,
};

#[test]
fn legacy_signatures_remain_available_in_1x() {
    let _: fn() -> Result<Config, ConfigError> = Config::new;
    let _: fn() -> Result<Database, DatabaseError> = Database::new;
    let _: fn(&Database, Option<String>) -> Result<Table, DatabaseError> = Table::new;
    let _: fn(&[u8]) -> Result<EncryptedData, chacha20poly1305::Error> = encrypt_data;
    let _: fn(u8, bool) -> String = generate_password;
    let _: fn(&[u8], &[u8]) -> Result<(String, String), String> = encrypt_with_key;
    let _: fn(&[u8], &str, &str) -> Result<String, String> = decrypt_with_key;
    let _: fn(&[u8]) -> Result<String, std::io::Error> = start_session;
    let _: fn() -> Result<Vec<u8>, String> = get_key_from_session;
    let _: fn() -> Result<(), String> = rvault_core::lock;

    let _: fn(&Table, &Database, String, String) = Table::add_entry;
    let _: fn(&Table, &Database, &[u8], String, String) = Table::add_entry_with_key;
    let _: fn(&Table, &Database, String, String) = Table::remove_entry;
    let _: fn(&Table, &Database, String, String) -> Result<(), DatabaseError> = Table::get_password;
    let _: fn(&Table, &Database, &[u8], String, String) -> Result<(), DatabaseError> =
        Table::get_password_with_key;

    let _ = std::mem::size_of::<Vault>();
    let _ = std::mem::size_of::<VaultEntry>();
    let _ = std::mem::size_of::<EncryptedData>();
    let _ = std::mem::size_of::<DerivedEncryptedData>();
    let _ = std::mem::size_of::<HashedData>();
    let _ = std::mem::size_of::<DatabaseError>();
    let _ = std::mem::size_of::<ConfigError>();
}
