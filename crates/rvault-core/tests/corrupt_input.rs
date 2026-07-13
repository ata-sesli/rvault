#![allow(deprecated)]

use rvault_core::crypto::{decrypt_with_key, generate_password};

fn assert_decrypt_error_without_panic(key: &[u8], ciphertext: &str, nonce: &str) {
    let outcome = std::panic::catch_unwind(|| decrypt_with_key(key, ciphertext, nonce));
    assert!(outcome.is_ok(), "malformed input must not panic");
    assert!(outcome.unwrap().is_err(), "malformed input must return Err");
}

#[test]
fn decrypt_with_key_rejects_invalid_base64_without_panicking() {
    assert_decrypt_error_without_panic(&[7_u8; 32], "%%%", "AA==");
    assert_decrypt_error_without_panic(&[7_u8; 32], "AA==", "%%%");
}

#[test]
fn decrypt_with_key_rejects_invalid_nonce_length_without_panicking() {
    assert_decrypt_error_without_panic(&[7_u8; 32], "AA==", "AA==");
    assert_decrypt_error_without_panic(&[7_u8; 32], "AA==", "AAAAAAAAAAAAAAAAAA==");
}

#[test]
fn decrypt_with_key_rejects_invalid_key_length_without_panicking() {
    assert_decrypt_error_without_panic(&[7_u8; 31], "AA==", "AAAAAAAAAAAAAAAA");
}

#[test]
fn generate_password_rejects_lengths_below_required_classes_without_panicking() {
    for length in 0..3 {
        let outcome = std::panic::catch_unwind(|| generate_password(length, false));
        assert!(outcome.is_ok(), "length {length} must not panic");
        assert!(outcome.unwrap().is_empty());
    }
    for length in 0..4 {
        let outcome = std::panic::catch_unwind(|| generate_password(length, true));
        assert!(outcome.is_ok(), "length {length} must not panic");
        assert!(outcome.unwrap().is_empty());
    }
}
