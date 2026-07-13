use thiserror::Error;

/// Failures returned by the typed cryptography API.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CryptoError {
    #[error("invalid key length: expected 32 bytes, got {actual}")]
    InvalidKeyLength { actual: usize },
    #[error("invalid nonce length: expected 12 bytes, got {actual}")]
    InvalidNonceLength { actual: usize },
    #[error("invalid encoded cryptographic data")]
    InvalidEncoding(#[source] base64::DecodeError),
    #[error("ciphertext authentication failed")]
    AuthenticationFailed,
    #[error("password length {length} is below the required minimum {minimum}")]
    InvalidPasswordLength { length: u8, minimum: u8 },
    #[error("key derivation failed: {0}")]
    KeyDerivation(String),
}
