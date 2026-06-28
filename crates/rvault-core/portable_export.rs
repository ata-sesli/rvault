use crate::{
    binary::{EXPORT_MAGIC, decode_envelope, encode_envelope},
    crypto::{decrypt_bytes_with_key, encrypt_bytes_with_key},
    identity::{IdentityKeypair, parse_public_code},
};
use hkdf::Hkdf;
use sha2::Sha256;
use x25519_dalek::{EphemeralSecret, PublicKey, StaticSecret};

const EXPORT_PAYLOAD_MAGIC: &[u8; 8] = b"RVEXPAY1";
const EXPORT_HKDF_SALT: &[u8] = b"rvault-export-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportEntry {
    pub platform: String,
    pub user_id: String,
    pub password: String,
    pub pinned: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

pub fn create_export_bytes(
    recipient_public_code: &str,
    entries: &[ExportEntry],
) -> Result<Vec<u8>, String> {
    let recipient_public_key = parse_public_code(recipient_public_code)?;
    let recipient_public = PublicKey::from(recipient_public_key);
    let ephemeral_secret = EphemeralSecret::random_from_rng(rand_core::OsRng);
    let ephemeral_public = PublicKey::from(&ephemeral_secret);
    let shared = ephemeral_secret.diffie_hellman(&recipient_public);
    let key = derive_export_key(
        shared.as_bytes(),
        ephemeral_public.as_bytes(),
        &recipient_public_key,
    )?;
    let payload = encode_export_payload(entries)?;
    let (nonce, ciphertext) = encrypt_bytes_with_key(&key, &payload)?;
    Ok(encode_envelope(
        EXPORT_MAGIC,
        &[
            ephemeral_public.as_bytes().to_vec(),
            recipient_public_key.to_vec(),
            nonce.to_vec(),
            ciphertext,
        ],
    ))
}

pub fn decrypt_export_bytes(
    identity: &IdentityKeypair,
    bytes: &[u8],
) -> Result<Vec<ExportEntry>, String> {
    let envelope = decode_envelope(bytes, EXPORT_MAGIC)?;
    if envelope.fields.len() != 4 {
        return Err("invalid export envelope field count".to_string());
    }
    let ephemeral_public: [u8; 32] = envelope.fields[0]
        .clone()
        .try_into()
        .map_err(|_| "invalid export ephemeral key length".to_string())?;
    let recipient_public: [u8; 32] = envelope.fields[1]
        .clone()
        .try_into()
        .map_err(|_| "invalid export recipient key length".to_string())?;
    if recipient_public != identity.public_key {
        return Err("export is encrypted for a different recipient".to_string());
    }
    let secret = StaticSecret::from(identity.private_key);
    let shared = secret.diffie_hellman(&PublicKey::from(ephemeral_public));
    let key = derive_export_key(shared.as_bytes(), &ephemeral_public, &recipient_public)?;
    let payload = decrypt_bytes_with_key(&key, &envelope.fields[2], &envelope.fields[3])?;
    decode_export_payload(&payload)
}

fn derive_export_key(
    shared_secret: &[u8],
    ephemeral_public: &[u8],
    recipient_public: &[u8],
) -> Result<[u8; 32], String> {
    let hk = Hkdf::<Sha256>::new(Some(EXPORT_HKDF_SALT), shared_secret);
    let mut info = Vec::with_capacity(ephemeral_public.len() + recipient_public.len());
    info.extend_from_slice(ephemeral_public);
    info.extend_from_slice(recipient_public);
    let mut key = [0_u8; 32];
    hk.expand(&info, &mut key)
        .map_err(|e| format!("derive export key: {e}"))?;
    Ok(key)
}

fn encode_export_payload(entries: &[ExportEntry]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    out.extend_from_slice(EXPORT_PAYLOAD_MAGIC);
    let count: u32 = entries
        .len()
        .try_into()
        .map_err(|_| "too many export entries".to_string())?;
    out.extend_from_slice(&count.to_le_bytes());
    for entry in entries {
        push_string(&mut out, &entry.platform)?;
        push_string(&mut out, &entry.user_id)?;
        push_string(&mut out, &entry.password)?;
        out.push(u8::from(entry.pinned));
        out.extend_from_slice(&entry.created_at.to_le_bytes());
        out.extend_from_slice(&entry.updated_at.to_le_bytes());
    }
    Ok(out)
}

fn decode_export_payload(bytes: &[u8]) -> Result<Vec<ExportEntry>, String> {
    if bytes.len() < EXPORT_PAYLOAD_MAGIC.len() + 4 {
        return Err("truncated export payload".to_string());
    }
    if &bytes[..EXPORT_PAYLOAD_MAGIC.len()] != EXPORT_PAYLOAD_MAGIC {
        return Err("invalid export payload magic".to_string());
    }
    let mut cursor = EXPORT_PAYLOAD_MAGIC.len();
    let count = read_u32(bytes, &mut cursor)? as usize;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let platform = read_string(bytes, &mut cursor)?;
        let user_id = read_string(bytes, &mut cursor)?;
        let password = read_string(bytes, &mut cursor)?;
        if bytes.len().saturating_sub(cursor) < 17 {
            return Err("truncated export payload entry".to_string());
        }
        let pinned = bytes[cursor] == 1;
        cursor += 1;
        let created_at = read_i64(bytes, &mut cursor)?;
        let updated_at = read_i64(bytes, &mut cursor)?;
        entries.push(ExportEntry {
            platform,
            user_id,
            password,
            pinned,
            created_at,
            updated_at,
        });
    }
    if cursor != bytes.len() {
        return Err("export payload has trailing bytes".to_string());
    }
    Ok(entries)
}

fn push_string(out: &mut Vec<u8>, value: &str) -> Result<(), String> {
    let bytes = value.as_bytes();
    let len: u32 = bytes
        .len()
        .try_into()
        .map_err(|_| "export string is too large".to_string())?;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(bytes);
    Ok(())
}

fn read_string(bytes: &[u8], cursor: &mut usize) -> Result<String, String> {
    let len = read_u32(bytes, cursor)? as usize;
    if bytes.len().saturating_sub(*cursor) < len {
        return Err("truncated export string".to_string());
    }
    let value = String::from_utf8(bytes[*cursor..*cursor + len].to_vec())
        .map_err(|e| format!("export string is not UTF-8: {e}"))?;
    *cursor += len;
    Ok(value)
}

fn read_u32(bytes: &[u8], cursor: &mut usize) -> Result<u32, String> {
    if bytes.len().saturating_sub(*cursor) < 4 {
        return Err("truncated export payload integer".to_string());
    }
    let mut out = [0_u8; 4];
    out.copy_from_slice(&bytes[*cursor..*cursor + 4]);
    *cursor += 4;
    Ok(u32::from_le_bytes(out))
}

fn read_i64(bytes: &[u8], cursor: &mut usize) -> Result<i64, String> {
    if bytes.len().saturating_sub(*cursor) < 8 {
        return Err("truncated export payload timestamp".to_string());
    }
    let mut out = [0_u8; 8];
    out.copy_from_slice(&bytes[*cursor..*cursor + 8]);
    *cursor += 8;
    Ok(i64::from_le_bytes(out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{
        generate_identity_bytes, load_identity_from_bytes, public_code_from_key,
    };

    const RECIPIENT_KEY: [u8; 32] = [9; 32];
    const WRONG_KEY: [u8; 32] = [3; 32];

    fn entries() -> Vec<ExportEntry> {
        vec![ExportEntry {
            platform: "Gmail".to_string(),
            user_id: "ata@example.com".to_string(),
            password: "secret-password".to_string(),
            pinned: false,
            created_at: 10,
            updated_at: 20,
        }]
    }

    #[test]
    fn export_bytes_round_trip_to_recipient_identity() {
        let identity_bytes = generate_identity_bytes(&RECIPIENT_KEY).expect("identity");
        let identity =
            load_identity_from_bytes(&RECIPIENT_KEY, &identity_bytes).expect("load identity");
        let public_code = public_code_from_key(&identity.public_key);

        let export = create_export_bytes(&public_code, &entries()).expect("create export");

        assert!(export.starts_with(crate::binary::EXPORT_MAGIC));
        assert!(!String::from_utf8_lossy(&export).contains("secret-password"));

        let imported = decrypt_export_bytes(&identity, &export).expect("decrypt export");

        assert_eq!(imported, entries());
    }

    #[test]
    fn export_bytes_reject_wrong_recipient() {
        let recipient_bytes = generate_identity_bytes(&RECIPIENT_KEY).expect("recipient identity");
        let recipient =
            load_identity_from_bytes(&RECIPIENT_KEY, &recipient_bytes).expect("load recipient");
        let wrong_bytes = generate_identity_bytes(&WRONG_KEY).expect("wrong identity");
        let wrong = load_identity_from_bytes(&WRONG_KEY, &wrong_bytes).expect("load wrong");
        let public_code = public_code_from_key(&recipient.public_key);
        let export = create_export_bytes(&public_code, &entries()).expect("create export");

        let err = decrypt_export_bytes(&wrong, &export).expect_err("wrong recipient should fail");

        assert!(err.contains("recipient") || err.contains("decrypt"));
    }
}
