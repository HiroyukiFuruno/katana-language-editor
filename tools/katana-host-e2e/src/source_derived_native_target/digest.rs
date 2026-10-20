use super::types::SourceDerivedNativeTargetError;

pub(super) const DIGEST_PREFIX: &str = "sha256:";
const SHA256_DIGEST_BYTES: usize = 32;
const SHA256_DIGEST_HEX_LENGTH: usize = SHA256_DIGEST_BYTES * 2;
const HEX_BYTE_LENGTH: usize = 2;
const HEX_DIGIT_BITS: u32 = 4;
const HEX_DECIMAL_OFFSET: u8 = 10;

pub(super) fn is_digest(value: &str) -> bool {
    value.strip_prefix(DIGEST_PREFIX).is_some_and(|hex| {
        hex.len() == SHA256_DIGEST_HEX_LENGTH && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
    })
}

pub(super) fn decode_digest(
    value: &str,
    field: &'static str,
) -> Result<[u8; SHA256_DIGEST_BYTES], SourceDerivedNativeTargetError> {
    let hex = value
        .strip_prefix(DIGEST_PREFIX)
        .ok_or(SourceDerivedNativeTargetError::InvalidDigest(field))?;
    let mut digest = [0_u8; SHA256_DIGEST_BYTES];
    for (index, chunk) in hex.as_bytes().chunks_exact(HEX_BYTE_LENGTH).enumerate() {
        digest[index] = (hex_digit(chunk[0])
            .ok_or(SourceDerivedNativeTargetError::InvalidDigest(field))?
            << HEX_DIGIT_BITS)
            | hex_digit(chunk[1]).ok_or(SourceDerivedNativeTargetError::InvalidDigest(field))?;
    }
    Ok(digest)
}

fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + HEX_DECIMAL_OFFSET),
        b'A'..=b'F' => Some(byte - b'A' + HEX_DECIMAL_OFFSET),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{SourceDerivedNativeTargetError, decode_digest};

    #[test]
    fn digest_decoder_returns_binary_sha256_bytes() -> Result<(), SourceDerivedNativeTargetError> {
        let value = decode_digest(
            "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "role_digest",
        )?;
        assert_eq!(value[0], 0x01);
        assert_eq!(value[31], 0xef);
        Ok(())
    }
}
