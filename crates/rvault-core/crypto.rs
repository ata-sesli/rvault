use argon2::{
    Argon2,
    password_hash::{self, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use base64::engine::general_purpose::STANDARD as Base64;
use base64::prelude::*;
use chacha20poly1305::{
    ChaCha20Poly1305, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use clap::ValueEnum;
use rand::seq::{IndexedRandom, SliceRandom};
use zeroize::Zeroizing;

mod error;

use crate::secret::{SecretBytes, SecretKey};
pub use error::CryptoError;
#[derive(Debug, Clone, ValueEnum)]
// Multiple encryption methods in future implementations
pub enum Encryption {
    Raw,
}
#[derive(Debug, Clone, ValueEnum)]
// Multiple hashing methods in future implementations
pub enum Hash {
    Raw,
}

pub struct EncryptedData {
    pub key: String,
    pub nonce: String,
    pub ciphertext: String,
}
pub struct DerivedEncryptedData {
    pub ciphertext: String,
    pub nonce: String,
    pub salt: String,
}
pub struct HashedData {
    pub hash: String,
}

/// Authenticated ciphertext produced by [`encrypt`].
pub struct Ciphertext {
    nonce: [u8; 12],
    bytes: Vec<u8>,
}

impl Ciphertext {
    /// Constructs ciphertext from persisted components after validating the nonce.
    pub fn try_from_parts(nonce: &[u8], bytes: Vec<u8>) -> Result<Self, CryptoError> {
        let nonce: [u8; 12] = nonce
            .try_into()
            .map_err(|_| CryptoError::InvalidNonceLength {
                actual: nonce.len(),
            })?;
        Ok(Self { nonce, bytes })
    }

    /// Returns the authenticated-encryption nonce.
    pub fn nonce(&self) -> &[u8; 12] {
        &self.nonce
    }

    /// Returns the encrypted bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Encrypts bytes with an opaque secret key.
pub fn encrypt(key: &SecretKey, plaintext: &[u8]) -> Result<Ciphertext, CryptoError> {
    let cipher = ChaCha20Poly1305::new_from_slice(key.as_bytes()).map_err(|_| {
        CryptoError::InvalidKeyLength {
            actual: key.as_bytes().len(),
        }
    })?;
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let bytes = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|_| CryptoError::AuthenticationFailed)?;
    Ciphertext::try_from_parts(nonce.as_slice(), bytes)
}

/// Decrypts authenticated ciphertext into zeroizing owned bytes.
pub fn decrypt(key: &SecretKey, ciphertext: &Ciphertext) -> Result<SecretBytes, CryptoError> {
    let cipher = ChaCha20Poly1305::new_from_slice(key.as_bytes()).map_err(|_| {
        CryptoError::InvalidKeyLength {
            actual: key.as_bytes().len(),
        }
    })?;
    cipher
        .decrypt(Nonce::from_slice(ciphertext.nonce()), ciphertext.bytes())
        .map(SecretBytes::new)
        .map_err(|_| CryptoError::AuthenticationFailed)
}

/// Generates a password or returns a typed length error.
pub fn try_generate_password(length: u8, special_characters: bool) -> Result<String, CryptoError> {
    let minimum = if special_characters { 4 } else { 3 };
    if length < minimum {
        return Err(CryptoError::InvalidPasswordLength { length, minimum });
    }
    Ok(generate_password(length, special_characters))
}
pub fn generate_key() -> String {
    let ek = ChaCha20Poly1305::generate_key(&mut OsRng); // A 32-byte random key
    let ek_b64 = Base64.encode(&ek);
    ek_b64
}
pub fn generate_raw_key() -> [u8; 32] {
    let key = ChaCha20Poly1305::generate_key(&mut OsRng);
    key.into()
}

// &[u8] means binary sequence.
pub fn hash_data(data: &[u8]) -> Result<HashedData, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(data, &salt)?.to_string();
    let hashed_data = HashedData { hash };
    Ok(hashed_data)
}

/// Derive a raw 32-byte encryption key from a password and salt using Argon2
/// Unlike `hash_data`, this returns the raw output bytes suitable for encryption keys
pub fn derive_key(password: &[u8], salt: &[u8]) -> Result<[u8; 32], argon2::Error> {
    let argon2 = Argon2::default();
    let mut output_key = Zeroizing::new([0u8; 32]);
    argon2.hash_password_into(password, salt, output_key.as_mut())?;
    Ok(*output_key)
}

pub fn encrypt_data(data: &[u8]) -> Result<EncryptedData, chacha20poly1305::Error> {
    let key = Zeroizing::new(<[u8; 32]>::from(ChaCha20Poly1305::generate_key(&mut OsRng)));
    let cipher = ChaCha20Poly1305::new_from_slice(key.as_ref()).expect("32-byte generated key");
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext_bytes = cipher.encrypt(&nonce, data)?;
    let encrypted_data = EncryptedData {
        key: Base64.encode(&key),
        nonce: Base64.encode(&nonce),
        ciphertext: Base64.encode(&ciphertext_bytes),
    };
    Ok(encrypted_data)
}
/// Generates a password containing each required character class.
///
/// For compatibility, this legacy API returns an empty string when `length` is smaller than the
/// number of required character classes (three, or four when special characters are requested).
pub fn generate_password(length: u8, special_characters: bool) -> String {
    let lowercase = b"abcdefghijklmnopqrstuvwxyz";
    let uppercase = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let numbers = b"0123456789";
    let symbols = b"!@#$%^&*()_+-=[]{}|;:'\",.<>/?";

    let required_classes = if special_characters { 4 } else { 3 };
    if length < required_classes {
        return String::new();
    }

    let mut rng = rand::rng();
    let mut password_chars = Vec::with_capacity(length as usize);
    let mut master_pool = Vec::new();

    master_pool.extend_from_slice(lowercase);
    password_chars.push(*lowercase.choose(&mut rng).unwrap() as char);

    master_pool.extend_from_slice(uppercase);
    password_chars.push(*uppercase.choose(&mut rng).unwrap() as char);

    master_pool.extend_from_slice(numbers);
    password_chars.push(*numbers.choose(&mut rng).unwrap() as char);

    if special_characters {
        master_pool.extend_from_slice(symbols);
        password_chars.push(*symbols.choose(&mut rng).unwrap() as char);
    }

    let remaining_len = length as usize - password_chars.len();
    for _ in 0..remaining_len {
        password_chars.push(*master_pool.choose(&mut rng).unwrap() as char);
    }
    password_chars.shuffle(&mut rng);
    password_chars.into_iter().collect::<String>()
}
pub fn encrypt_with_key(key: &[u8], data: &[u8]) -> Result<(String, String), String> {
    let cipher = ChaCha20Poly1305::new_from_slice(key).map_err(|e| e.to_string())?;
    let nonce = ChaCha20Poly1305::generate_nonce(&mut chacha20poly1305::aead::OsRng);
    let ciphertext = cipher.encrypt(&nonce, data).map_err(|e| e.to_string())?;

    Ok((Base64.encode(&ciphertext), Base64.encode(&nonce)))
}

pub fn encrypt_bytes_with_key(key: &[u8], data: &[u8]) -> Result<([u8; 12], Vec<u8>), String> {
    let cipher = ChaCha20Poly1305::new_from_slice(key).map_err(|e| e.to_string())?;
    let nonce = ChaCha20Poly1305::generate_nonce(&mut chacha20poly1305::aead::OsRng);
    let ciphertext = cipher.encrypt(&nonce, data).map_err(|e| e.to_string())?;
    let mut nonce_bytes = [0_u8; 12];
    nonce_bytes.copy_from_slice(nonce.as_slice());
    Ok((nonce_bytes, ciphertext))
}

pub fn decrypt_bytes_with_key(
    key: &[u8],
    nonce_bytes: &[u8],
    ciphertext: &[u8],
) -> Result<Vec<u8>, String> {
    if nonce_bytes.len() != 12 {
        return Err("invalid nonce length".to_string());
    }
    let cipher = ChaCha20Poly1305::new_from_slice(key).map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("decrypt failed: {e}"))
}

pub fn decrypt_with_key(
    key: &[u8],
    ciphertext_b64: &str,
    nonce_b64: &str,
) -> Result<String, String> {
    let ciphertext = Base64.decode(ciphertext_b64).map_err(|e| e.to_string())?;
    let nonce_bytes = Base64.decode(nonce_b64).map_err(|e| e.to_string())?;
    if nonce_bytes.len() != 12 {
        return Err("invalid nonce length".to_string());
    }
    let nonce = Nonce::from_slice(&nonce_bytes);
    let cipher = ChaCha20Poly1305::new_from_slice(key).map_err(|e| e.to_string())?;
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| e.to_string())?;
    String::from_utf8(plaintext).map_err(|e| e.to_string())
}
pub fn verify_password(password: &[u8], stored_hash: &str) -> bool {
    // Attempt to parse the stored hash string
    if let Ok(parsed_hash) = PasswordHash::new(stored_hash) {
        // Verify the plaintext password against the parsed hash
        Argon2::default()
            .verify_password(password, &parsed_hash)
            .is_ok() // Returns true if verification succeeds, false otherwise
    } else {
        // If the stored hash is invalid, verification fails
        false
    }
}

#[cfg(test)]
mod typed_api_tests {
    use super::*;
    use crate::secret::SecretKey;

    #[test]
    fn typed_api_rejects_invalid_nonce_length() {
        let error = match Ciphertext::try_from_parts(&[0_u8; 11], vec![1, 2, 3]) {
            Err(error) => error,
            Ok(_) => panic!("invalid nonce was accepted"),
        };
        assert!(matches!(
            error,
            CryptoError::InvalidNonceLength { actual: 11 }
        ));
    }

    #[test]
    fn typed_api_classifies_authentication_failure() {
        let ciphertext = encrypt(&SecretKey::from_bytes([1_u8; 32]), b"secret").unwrap();
        let error = match decrypt(&SecretKey::from_bytes([2_u8; 32]), &ciphertext) {
            Err(error) => error,
            Ok(_) => panic!("wrong key decrypted ciphertext"),
        };
        assert!(matches!(error, CryptoError::AuthenticationFailed));
    }

    #[test]
    fn typed_api_rejects_short_password_length() {
        let error = try_generate_password(2, false).unwrap_err();
        assert!(matches!(
            error,
            CryptoError::InvalidPasswordLength {
                length: 2,
                minimum: 3
            }
        ));
    }

    #[test]
    fn typed_api_crypto_round_trips_secret_bytes() {
        let key = SecretKey::from_bytes([7_u8; 32]);
        let ciphertext = encrypt(&key, b"round trip").unwrap();
        assert_eq!(ciphertext.nonce().len(), 12);
        assert!(!ciphertext.bytes().is_empty());
        assert_eq!(decrypt(&key, &ciphertext).unwrap().expose(), b"round trip");
        assert_eq!(try_generate_password(12, true).unwrap().len(), 12);
    }
}
