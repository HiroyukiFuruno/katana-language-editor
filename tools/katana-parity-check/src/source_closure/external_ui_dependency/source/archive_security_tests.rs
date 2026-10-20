use flate2::{Compression, write::GzEncoder};

use super::assert_rejected;

const PACKAGE: &str = "egui-0.36.1/";
const REGULAR_FILE_MODE: u32 = 0o644;

fn gzip(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    std::io::Write::write_all(&mut encoder, bytes).map_err(|error| error.to_string())?;
    encoder.finish().map_err(|error| error.to_string())
}

fn raw_tar(entries: &[(&str, u8, &[u8])]) -> Result<Vec<u8>, String> {
    let mut builder = tar::Builder::new(Vec::new());
    for (name, entry_type, body) in entries {
        let mut header = tar::Header::new_ustar();
        if name.len() > header.as_old_mut().name.len() {
            return Err("raw TAR fixture path is too long".into());
        }
        header.as_old_mut().name[..name.len()].copy_from_slice(name.as_bytes());
        header.set_mode(REGULAR_FILE_MODE);
        header.set_size(body.len() as u64);
        header.set_mtime(1);
        header.set_entry_type(tar::EntryType::new(*entry_type));
        header.set_cksum();
        builder
            .append(&header, *body)
            .map_err(|error| error.to_string())?;
    }
    builder.into_inner().map_err(|error| error.to_string())
}

fn assert_raw_rejected(
    entries: &[(&str, u8, &[u8])],
    paths: &[&str],
    reason: &str,
) -> Result<(), String> {
    assert_rejected(gzip(&raw_tar(entries)?)?, paths, reason)
}

#[test]
fn raw_directory_traversal_dot_absolute_and_double_slash_paths_are_rejected() -> Result<(), String>
{
    let cases = [
        ("egui-0.36.1/../", "invalid path"),
        ("egui-0.36.1/./", "invalid path"),
        ("/egui-0.36.1/src/", "invalid package prefix"),
        ("egui-0.36.1/src//", "invalid path"),
        ("egui-0.36.1//", "invalid path"),
    ];
    for (path, reason) in cases {
        assert_raw_rejected(&[(path, b'5', &[])], &[], reason)?;
    }
    Ok(())
}

#[test]
fn raw_hardlinks_are_rejected() -> Result<(), String> {
    assert_raw_rejected(
        &[("egui-0.36.1/src/lib.rs", b'1', &[])],
        &["src/lib.rs"],
        "unsupported non-regular entry",
    )
}

#[test]
fn checksum_valid_truncated_gzip_footer_is_rejected() -> Result<(), String> {
    let tar = raw_tar(&[(PACKAGE, b'5', &[]), ("egui-0.36.1/src/lib.rs", 0, b"x")])?;
    let mut bytes = gzip(&tar)?;
    if bytes.len() < 8 {
        return Err("gzip fixture has no footer".into());
    }
    bytes.truncate(bytes.len() - 8);
    assert_rejected(bytes, &["src/lib.rs"], "malformed")
}

#[test]
fn checksum_valid_bad_tar_is_rejected() -> Result<(), String> {
    assert_rejected(gzip(b"not a TAR archive")?, &[], "malformed")
}

#[test]
fn manifest_extra_rust_path_is_rejected() -> Result<(), String> {
    let tar = raw_tar(&[("egui-0.36.1/src/lib.rs", 0, b"x")])?;
    assert_rejected(
        gzip(&tar)?,
        &["src/lib.rs", "src/extra.rs"],
        "Rust paths differ",
    )
}
