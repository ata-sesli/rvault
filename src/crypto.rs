use clap::ValueEnum;
use rand::seq::{IndexedRandom, SliceRandom};
use chacha20poly1305::{aead::{Aead,AeadCore,KeyInit,OsRng}, ChaChaPoly1305,ChaCha20Poly1305};
use argon2::{password_hash::{self, rand_core::Error, PasswordHash, PasswordHasher, SaltString}, Argon2};
use base64::prelude::*;
use base64::engine::general_purpose::STANDARD as Base64;
#[derive(Debug,Clone,ValueEnum)]
// Multiple encryption methods in future implementations
pub enum Encryption {
    Raw,
    
}
#[derive(Debug,Clone,ValueEnum)]
// Multiple hashing methods in future implementations
pub enum Hash {
    Raw,
}

pub struct EncryptedData{
    key: String,
    nonce: String,
    ciphertext: String
}
pub struct HashedData{
    hash: String
}
// &[u8] means binary sequence.
pub fn hash_data(data:&[u8]) -> Result<HashedData,argon2::password_hash::Error>{
    let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(data, &salt)?.to_string();
    let hashed_data = HashedData{
        hash
    };
    Ok(hashed_data)
}
pub fn encrypt_data(data:&[u8]) -> Result<EncryptedData,chacha20poly1305::Error> {
    let key = ChaCha20Poly1305::generate_key(&mut OsRng);
    let cipher = ChaCha20Poly1305::new(&key);
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext_bytes = cipher.encrypt(&nonce, data)?;
    let encrypted_data = EncryptedData{
        key: Base64.encode(&key),
        nonce: Base64.encode(&nonce),
        ciphertext: Base64.encode(&ciphertext_bytes)
    };
    Ok(encrypted_data)
}
pub fn generate_password(length: u8,special_characters: bool) -> String{
    let lowercase = b"abcdefghijklmnopqrstuvwxyz";
    let uppercase = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let numbers = b"0123456789";
    let symbols = b"!@#$%^&*()_+-=[]{}|;:'\",.<>/?";

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