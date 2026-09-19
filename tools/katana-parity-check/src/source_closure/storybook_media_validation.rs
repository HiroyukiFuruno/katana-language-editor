use std::path::Path;

const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
const PNG_U32_FIELD_BYTES: usize = std::mem::size_of::<u32>();
const PNG_CHUNK_TYPE_BYTES: usize = [0u8; 4].len();
const PNG_CHUNK_DATA_OFFSET: usize = PNG_U32_FIELD_BYTES + PNG_CHUNK_TYPE_BYTES;
const PNG_CHUNK_OVERHEAD_BYTES: usize = PNG_CHUNK_DATA_OFFSET + PNG_U32_FIELD_BYTES;
const PNG_IHDR_DATA_BYTES: usize = 13;
const PNG_CRC32_POLYNOMIAL: u32 = 0xedb8_8320;

pub(super) fn is_numbered_stage_png(path: &Path, bytes: &[u8]) -> bool {
    filename_has_stage_ordinal(path) && is_structurally_valid_png(bytes)
}

fn filename_has_stage_ordinal(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .and_then(|filename| filename.strip_suffix(".png"))
        .and_then(|stem| stem.rsplit_once('-'))
        .is_some_and(|(_, ordinal)| {
            !ordinal.is_empty() && ordinal.bytes().all(|byte| byte.is_ascii_digit())
        })
}

fn is_structurally_valid_png(bytes: &[u8]) -> bool {
    if !bytes.starts_with(&PNG_SIGNATURE) {
        return false;
    }
    let mut remaining = &bytes[PNG_SIGNATURE.len()..];
    let mut saw_ihdr = false;
    let mut saw_idat = false;
    while !remaining.is_empty() {
        let Some((chunk_length, chunk_type, payload, checksum, tail)) = parse_chunk(remaining)
        else {
            return false;
        };
        if checksum
            != png_crc32(&remaining[PNG_U32_FIELD_BYTES..PNG_CHUNK_DATA_OFFSET + chunk_length])
        {
            return false;
        }
        match chunk_type {
            value if value == *b"IHDR" && !saw_ihdr && chunk_length == PNG_IHDR_DATA_BYTES => {
                saw_ihdr = true;
            }
            value if value == *b"IDAT" && saw_ihdr => saw_idat = true,
            value if value == *b"IEND" && saw_ihdr && saw_idat && payload.is_empty() => {
                return tail.is_empty();
            }
            _ => return false,
        }
        remaining = tail;
    }
    false
}

type PngChunk<'a> = (usize, [u8; PNG_CHUNK_TYPE_BYTES], &'a [u8], u32, &'a [u8]);

fn parse_chunk(bytes: &[u8]) -> Option<PngChunk<'_>> {
    let length = usize::try_from(u32::from_be_bytes(
        bytes.get(..PNG_U32_FIELD_BYTES)?.try_into().ok()?,
    ))
    .ok()?;
    let end = PNG_CHUNK_OVERHEAD_BYTES.checked_add(length)?;
    if bytes.len() < end {
        return None;
    }
    let chunk_type = bytes
        .get(PNG_U32_FIELD_BYTES..PNG_CHUNK_DATA_OFFSET)?
        .try_into()
        .ok()?;
    let payload = bytes.get(PNG_CHUNK_DATA_OFFSET..PNG_CHUNK_DATA_OFFSET + length)?;
    let checksum = u32::from_be_bytes(
        bytes
            .get(PNG_CHUNK_DATA_OFFSET + length..end)?
            .try_into()
            .ok()?,
    );
    Some((length, chunk_type, payload, checksum, &bytes[end..]))
}

fn png_crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..u8::BITS {
            crc = if crc & 1 == 0 {
                crc >> 1
            } else {
                (crc >> 1) ^ PNG_CRC32_POLYNOMIAL
            };
        }
    }
    !crc
}
