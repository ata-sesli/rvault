use serde::{Deserialize, Serialize};

use crate::{config::Config, crypto::{generate_key, hash_data, verify_password}, storage::{Database, Table}};
use base64::prelude::*;
use base64::engine::general_purpose::STANDARD as Base64;

const KEYRING_SERVICE: &str = "RVault";
const KEYRING_ACCOUNT: &str = "encryption_key"; // stable account name
pub struct Vault {
    vault: Vec<VaultEntry>,
    db: Database,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct VaultEntry {
    pub platform: String,
    pub user_id: String,
    pub password: String,
    pub salt: Option<String>,
    pub nonce: Option<String>
}
impl Vault {
    pub fn get_encryption_key(master_password: &str, stored_master_hash: &str) -> Result<[u8; 32], String> {
        if !crate::crypto::verify_password(master_password.as_bytes(), stored_master_hash) {
            return Err("Invalid master password".into());
        }
        let path = super::keystore_path()?; // or reimplement helper here
        crate::keystore::load_key_from_vault(master_password, &path)
    }
    fn encrypt_vault(){}
    fn encrypt_partial_vault(){}
    pub fn export_vault(){}
    pub fn export_partial_vault(){}
}