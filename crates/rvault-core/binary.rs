pub const BACKUP_MAGIC: &[u8; 4] = b"RVBK";
pub const EXPORT_MAGIC: &[u8; 4] = b"RVEX";
pub const IDENTITY_MAGIC: &[u8; 4] = b"RVID";
pub const ENVELOPE_VERSION: u8 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryEnvelope {
    pub magic: [u8; 4],
    pub version: u8,
    pub fields: Vec<Vec<u8>>,
}

pub fn encode_envelope(magic: &[u8; 4], fields: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(magic);
    out.push(ENVELOPE_VERSION);
    out.extend_from_slice(&(fields.len() as u32).to_le_bytes());
    for field in fields {
        out.extend_from_slice(&(field.len() as u64).to_le_bytes());
        out.extend_from_slice(field);
    }
    out
}

pub fn decode_envelope(bytes: &[u8], expected_magic: &[u8; 4]) -> Result<BinaryEnvelope, String> {
    if bytes.len() < 9 {
        return Err("truncated RVault binary envelope".to_string());
    }

    let mut magic = [0_u8; 4];
    magic.copy_from_slice(&bytes[..4]);
    if &magic != expected_magic {
        return Err("invalid RVault binary envelope magic".to_string());
    }

    let version = bytes[4];
    if version != ENVELOPE_VERSION {
        return Err(format!(
            "unsupported RVault binary envelope version: {version}"
        ));
    }

    let mut field_count_bytes = [0_u8; 4];
    field_count_bytes.copy_from_slice(&bytes[5..9]);
    let field_count = u32::from_le_bytes(field_count_bytes) as usize;
    let mut cursor = 9;
    let mut fields = Vec::with_capacity(field_count);

    for _ in 0..field_count {
        if bytes.len().saturating_sub(cursor) < 8 {
            return Err("truncated RVault binary envelope field length".to_string());
        }
        let mut len_bytes = [0_u8; 8];
        len_bytes.copy_from_slice(&bytes[cursor..cursor + 8]);
        cursor += 8;
        let len = u64::from_le_bytes(len_bytes) as usize;
        if bytes.len().saturating_sub(cursor) < len {
            return Err("truncated RVault binary envelope field".to_string());
        }
        fields.push(bytes[cursor..cursor + len].to_vec());
        cursor += len;
    }

    if cursor != bytes.len() {
        return Err("RVault binary envelope has trailing bytes".to_string());
    }

    Ok(BinaryEnvelope {
        magic,
        version,
        fields,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_envelope_round_trips_fields() {
        let bytes = encode_envelope(BACKUP_MAGIC, &[b"salt".to_vec(), b"cipher".to_vec()]);

        let decoded = decode_envelope(&bytes, BACKUP_MAGIC).expect("decode envelope");

        assert_eq!(decoded.magic, *BACKUP_MAGIC);
        assert_eq!(decoded.version, ENVELOPE_VERSION);
        assert_eq!(decoded.fields, vec![b"salt".to_vec(), b"cipher".to_vec()]);
    }

    #[test]
    fn binary_envelope_rejects_wrong_magic() {
        let bytes = encode_envelope(EXPORT_MAGIC, &[b"payload".to_vec()]);

        let err = decode_envelope(&bytes, BACKUP_MAGIC).expect_err("wrong magic should fail");

        assert!(err.contains("magic"));
    }

    #[test]
    fn binary_envelope_rejects_truncated_field() {
        let mut bytes = encode_envelope(BACKUP_MAGIC, &[b"payload".to_vec()]);
        bytes.pop();

        let err = decode_envelope(&bytes, BACKUP_MAGIC).expect_err("truncated field should fail");

        assert!(err.contains("truncated"));
    }
}
