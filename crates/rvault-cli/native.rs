use std::io::{Read, Write};

const LENGTH_PREFIX_BYTES: usize = 4;

pub fn encode_message_bytes(message: &[u8]) -> Result<Vec<u8>, String> {
    let len: u32 = message
        .len()
        .try_into()
        .map_err(|_| "message too large for native messaging frame".to_string())?;
    let mut frame = Vec::with_capacity(LENGTH_PREFIX_BYTES + message.len());
    frame.extend_from_slice(&len.to_le_bytes());
    frame.extend_from_slice(message);
    Ok(frame)
}

#[cfg(test)]
pub fn decode_message_bytes(frame: &[u8]) -> Result<Vec<u8>, String> {
    if frame.len() < LENGTH_PREFIX_BYTES {
        return Err("truncated native messaging length prefix".to_string());
    }

    let mut len_bytes = [0_u8; LENGTH_PREFIX_BYTES];
    len_bytes.copy_from_slice(&frame[..LENGTH_PREFIX_BYTES]);
    let len = u32::from_le_bytes(len_bytes) as usize;
    let end = LENGTH_PREFIX_BYTES + len;

    if frame.len() < end {
        return Err("truncated native messaging frame".to_string());
    }

    Ok(frame[LENGTH_PREFIX_BYTES..end].to_vec())
}

pub fn read_message<R: Read>(mut input: R) -> Result<String, String> {
    let mut len_bytes = [0_u8; LENGTH_PREFIX_BYTES];
    input
        .read_exact(&mut len_bytes)
        .map_err(|e| format!("failed to read native message length: {e}"))?;
    let len = u32::from_le_bytes(len_bytes) as usize;
    let mut payload = vec![0_u8; len];
    input
        .read_exact(&mut payload)
        .map_err(|e| format!("failed to read native message payload: {e}"))?;
    String::from_utf8(payload).map_err(|e| format!("native message is not UTF-8: {e}"))
}

pub fn write_message<W: Write>(mut output: W, message: &str) -> Result<(), String> {
    let frame = encode_message_bytes(message.as_bytes())?;
    output
        .write_all(&frame)
        .map_err(|e| format!("failed to write native message: {e}"))?;
    output
        .flush()
        .map_err(|e| format!("failed to flush native message: {e}"))
}

pub fn serve_stdio() -> Result<(), String> {
    let request = read_message(std::io::stdin().lock())?;
    let response = crate::extension_api::handle_request_json(&request);
    write_message(std::io::stdout().lock(), &response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_native_message_with_little_endian_length_prefix() {
        let frame = encode_message_bytes(br#"{"type":"status"}"#).expect("encode frame");

        assert_eq!(&frame[..4], &(17_u32).to_le_bytes());
        assert_eq!(&frame[4..], br#"{"type":"status"}"#);
    }

    #[test]
    fn decodes_native_message_frame() {
        let frame =
            encode_message_bytes(br#"{"type":"generate","length":8}"#).expect("encode frame");

        let decoded = decode_message_bytes(&frame).expect("decode frame");

        assert_eq!(decoded, br#"{"type":"generate","length":8}"#);
    }

    #[test]
    fn rejects_truncated_native_message_frame() {
        let frame = vec![10, 0, 0, 0, b'{'];

        let err = decode_message_bytes(&frame).expect_err("truncated frame should fail");

        assert!(err.contains("truncated"));
    }
}
