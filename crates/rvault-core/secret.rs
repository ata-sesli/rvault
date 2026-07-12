use zeroize::Zeroizing;

/// An owned 256-bit key that clears its memory on drop.
///
/// Secret keys intentionally cannot be formatted:
///
/// ```compile_fail
/// use rvault_core::secret::SecretKey;
/// let key = SecretKey::from_bytes([0_u8; 32]);
/// println!("{key:?}");
/// ```
///
/// ```compile_fail
/// use rvault_core::secret::SecretKey;
/// let key = SecretKey::from_bytes([0_u8; 32]);
/// println!("{key}");
/// ```
pub struct SecretKey(Zeroizing<[u8; 32]>);

impl SecretKey {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(Zeroizing::new(bytes))
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Owned secret bytes that clear their memory on drop.
pub struct SecretBytes(Zeroizing<Vec<u8>>);

impl SecretBytes {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(Zeroizing::new(bytes))
    }

    pub fn expose(&self) -> &[u8] {
        &self.0
    }
}
