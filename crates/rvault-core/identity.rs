use crate::{
    binary::{IDENTITY_MAGIC, decode_envelope, encode_envelope},
    crypto::{decrypt_bytes_with_key, encrypt_bytes_with_key},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use directories::ProjectDirs;
use std::{fs, path::PathBuf};
use x25519_dalek::{PublicKey, StaticSecret};

const PUBLIC_CODE_PREFIX: &str = "rvault1-";
const IDENTITY_NAME: &str = "identity.rvault";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityKeypair {
    pub private_key: [u8; 32],
    pub public_key: [u8; 32],
}

pub fn generate_identity_bytes(encryption_key: &[u8]) -> Result<Vec<u8>, String> {
    let private = StaticSecret::random_from_rng(rand_core::OsRng);
    let private_bytes = private.to_bytes();
    let (nonce, ciphertext) = encrypt_bytes_with_key(encryption_key, &private_bytes)?;
    Ok(encode_envelope(
        IDENTITY_MAGIC,
        &[nonce.to_vec(), ciphertext],
    ))
}

pub fn load_identity_from_bytes(
    encryption_key: &[u8],
    bytes: &[u8],
) -> Result<IdentityKeypair, String> {
    let envelope = decode_envelope(bytes, IDENTITY_MAGIC)?;
    if envelope.fields.len() != 2 {
        return Err("invalid identity envelope field count".to_string());
    }
    let private = decrypt_bytes_with_key(encryption_key, &envelope.fields[0], &envelope.fields[1])?;
    let private_key: [u8; 32] = private
        .try_into()
        .map_err(|_| "invalid identity private key length".to_string())?;
    let secret = StaticSecret::from(private_key);
    let public_key = PublicKey::from(&secret).to_bytes();
    Ok(IdentityKeypair {
        private_key,
        public_key,
    })
}

pub fn public_code_from_key(public_key: &[u8; 32]) -> String {
    format!("{PUBLIC_CODE_PREFIX}{}", URL_SAFE_NO_PAD.encode(public_key))
}

pub fn parse_public_code(code: &str) -> Result<[u8; 32], String> {
    let encoded = code
        .strip_prefix(PUBLIC_CODE_PREFIX)
        .ok_or_else(|| "invalid RVault public code prefix".to_string())?;
    let bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|e| format!("invalid RVault public code: {e}"))?;
    bytes
        .try_into()
        .map_err(|_| "invalid RVault public key length".to_string())
}

pub fn identity_path() -> Result<PathBuf, String> {
    if let Some(pd) = ProjectDirs::from("io.github", "ata-sesli", "RVault") {
        let dir = pd.config_dir();
        fs::create_dir_all(dir).map_err(|e| format!("mkdir identity dir: {e}"))?;
        Ok(dir.join(IDENTITY_NAME))
    } else {
        Err("Could not find project directories".to_string())
    }
}

pub fn load_or_create_identity(encryption_key: &[u8]) -> Result<IdentityKeypair, String> {
    let path = identity_path()?;
    if path.exists() {
        let bytes = fs::read(&path).map_err(|e| format!("read identity: {e}"))?;
        return load_identity_from_bytes(encryption_key, &bytes);
    }
    let bytes = generate_identity_bytes(encryption_key)?;
    fs::write(&path, &bytes).map_err(|e| format!("write identity: {e}"))?;
    load_identity_from_bytes(encryption_key, &bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    const VAULT_KEY: [u8; 32] = [7; 32];

    #[test]
    fn identity_bytes_round_trip_without_exposing_private_key() {
        let bytes = generate_identity_bytes(&VAULT_KEY).expect("generate identity");

        assert!(bytes.starts_with(crate::binary::IDENTITY_MAGIC));
        assert!(!bytes.windows(32).any(|window| window == [0; 32]));

        let identity = load_identity_from_bytes(&VAULT_KEY, &bytes).expect("load identity");
        let public_code = public_code_from_key(&identity.public_key);
        let parsed = parse_public_code(&public_code).expect("parse public code");

        assert_eq!(parsed, identity.public_key);
        assert!(public_code.starts_with("rvault1-"));
    }

    #[test]
    fn identity_bytes_reject_wrong_vault_key() {
        let bytes = generate_identity_bytes(&VAULT_KEY).expect("generate identity");

        let err = load_identity_from_bytes(&[8; 32], &bytes)
            .expect_err("wrong key should not load identity");

        assert!(err.contains("decrypt"));
    }
}
