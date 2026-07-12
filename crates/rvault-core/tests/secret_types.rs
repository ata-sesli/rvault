use rvault_core::secret::{SecretBytes, SecretKey};

#[test]
fn secret_types_expose_bytes_only_by_explicit_borrowing() {
    let key = SecretKey::from_bytes([7_u8; 32]);
    assert_eq!(key.as_bytes(), &[7_u8; 32]);

    let bytes = SecretBytes::new(vec![1, 2, 3]);
    assert_eq!(bytes.expose(), &[1, 2, 3]);
}
