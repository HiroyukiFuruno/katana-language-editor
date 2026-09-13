use std::io::{Cursor, Read};

use super::{CappedReader, MAX_DECOMPRESSED_BYTES};

#[test]
fn decompressed_limit_accepts_exact_eof_and_rejects_the_next_byte() -> std::io::Result<()> {
    let mut exact = CappedReader {
        reader: Cursor::new(b"x"),
        read: MAX_DECOMPRESSED_BYTES - 1,
    };
    let mut byte = [0];
    assert_eq!(exact.read(&mut byte)?, 1);
    assert_eq!(exact.read(&mut byte)?, 0);
    let mut oversized = CappedReader {
        reader: Cursor::new(b"xy"),
        read: MAX_DECOMPRESSED_BYTES - 1,
    };
    assert_eq!(oversized.read(&mut byte)?, 1);
    assert_eq!(oversized.read(&mut [])?, 0);
    assert_eq!(oversized.reader.position(), 1);
    let error = oversized
        .read(&mut byte)
        .err()
        .ok_or_else(|| std::io::Error::other("oversized decompressed stream was accepted"))?;
    assert!(error.to_string().contains("256MiB decompressed limit"));
    Ok(())
}
