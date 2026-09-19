use std::fs;
use std::io::Cursor;

use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde_json::json;
use tar::{Archive, Builder, EntryType, Header};

use super::super::super::super::capability_manifest::capability_manifest_test_fixtures::FixtureBuilder;
use super::super::super::fingerprint::sha256_hex;
use super::super::lock::LockPackage;
use super::archive::rust_file_hashes;

const REGULAR_FILE_MODE: u32 = 0o644;
const FIXTURE_MTIME: u64 = 1;

fn lock(bytes: &[u8]) -> LockPackage {
    LockPackage {
        version: "0.36.1".into(),
        checksum: sha256_hex(bytes).trim_start_matches("sha256:").into(),
        ..LockPackage::default()
    }
}

fn manifest(paths: &[&str]) -> serde_json::Map<String, serde_json::Value> {
    paths
        .iter()
        .map(|path| ((*path).into(), json!("unused")))
        .collect()
}

fn archive(entries: &[(&str, &str)]) -> Result<Vec<u8>, String> {
    let mut tar = Builder::new(GzEncoder::new(Vec::new(), Compression::default()));
    for (name, body) in entries {
        let mut header = Header::new_gnu();
        header.set_size(body.len() as u64);
        header.set_mode(REGULAR_FILE_MODE);
        header.set_mtime(FIXTURE_MTIME);
        header.set_cksum();
        tar.append_data(&mut header, name, Cursor::new(body.as_bytes()))
            .map_err(|error| error.to_string())?;
    }
    tar.into_inner()
        .map_err(|error| error.to_string())?
        .finish()
        .map_err(|error| error.to_string())
}

#[test]
fn gnu_directory_entries_and_nonzero_timestamps_are_valid() -> Result<(), String> {
    let fixture = FixtureBuilder::root()?;
    let path = fixture.path().join("egui-0.36.1.crate");
    let mut tar = Builder::new(GzEncoder::new(Vec::new(), Compression::default()));
    for name in ["egui-0.36.1/", "egui-0.36.1/src/"] {
        let mut header = Header::new_gnu();
        header.set_entry_type(EntryType::Directory);
        header.set_size(0);
        header.set_mtime(FIXTURE_MTIME);
        header.set_cksum();
        tar.append_data(&mut header, name, Cursor::new([]))
            .map_err(|error| error.to_string())?;
    }
    let mut header = Header::new_gnu();
    header.set_size(1);
    header.set_mode(REGULAR_FILE_MODE);
    header.set_mtime(FIXTURE_MTIME);
    header.set_cksum();
    tar.append_data(&mut header, "egui-0.36.1/src/lib.rs", Cursor::new(b"x"))
        .map_err(|error| error.to_string())?;
    let bytes = tar
        .into_inner()
        .map_err(|error| error.to_string())?
        .finish()
        .map_err(|error| error.to_string())?;
    fs::write(&path, &bytes).map_err(|error| error.to_string())?;
    rust_file_hashes(&path, &lock(&bytes), &manifest(&["src/lib.rs"]))
        .map_err(|error| format!("directory fixture rejected: {error}"))?;
    Ok(())
}

fn assert_rejected(bytes: Vec<u8>, paths: &[&str], reason: &str) -> Result<(), String> {
    let fixture = FixtureBuilder::root()?;
    let path = fixture.path().join("egui-0.36.1.crate");
    fs::write(&path, &bytes).map_err(|error| error.to_string())?;
    let error = match rust_file_hashes(&path, &lock(&bytes), &manifest(paths)) {
        Ok(_) => return Err("archive unexpectedly passed validation".into()),
        Err(error) => error,
    };
    assert!(error.contains(reason), "expected {reason:?}, got {error:?}");
    Ok(())
}

#[test]
fn missing_tampered_and_malformed_archives_are_rejected() -> Result<(), String> {
    let fixture = FixtureBuilder::root()?;
    let missing = fixture.path().join("missing.crate");
    assert!(rust_file_hashes(&missing, &lock(b"x"), &manifest(&[])).is_err());
    let valid = archive(&[("egui-0.36.1/src/lib.rs", "pub fn x() {}")])?;
    let mut tampered = valid.clone();
    tampered[0] ^= 1;
    let path = fixture.path().join("tampered.crate");
    fs::write(&path, tampered).map_err(|error| error.to_string())?;
    let error = rust_file_hashes(&path, &lock(&valid), &manifest(&["src/lib.rs"]))
        .err()
        .ok_or_else(|| "tampered archive unexpectedly passed validation".to_owned())?;
    assert!(error.contains("differs from Cargo.lock checksum"));
    assert_rejected(b"not gzip".to_vec(), &[], "malformed")?;
    assert_rejected(
        archive(&[("egui-0.36.1/src/lib.rs", "broken")])?,
        &[],
        "Rust paths differ",
    )
}

#[test]
fn archive_path_and_duplicate_failures_are_rejected() -> Result<(), String> {
    for (entries, reason) in [
        (
            vec![("other-0.36.1/src/lib.rs", "x")],
            "invalid package prefix",
        ),
        (vec![("egui-0.36.1/C:lib.rs", "x")], "invalid path"),
        (vec![("egui-0.36.1/src\\lib.rs", "x")], "invalid path"),
        (
            vec![
                ("egui-0.36.1/src/lib.rs", "x"),
                ("egui-0.36.1/src/lib.rs", "y"),
            ],
            "duplicate path",
        ),
    ] {
        assert_rejected(archive(&entries)?, &["src/lib.rs"], reason)?;
    }
    Ok(())
}

#[test]
fn raw_backslash_header_is_not_normalized_into_success() -> Result<(), String> {
    let bytes = archive(&[("egui-0.36.1/src\\lib.rs", "x")])?;
    let mut tar = Archive::new(GzDecoder::new(bytes.as_slice()));
    let entry = tar
        .entries()
        .map_err(|error| error.to_string())?
        .next()
        .ok_or_else(|| "fixture has no entry".to_owned())?
        .map_err(|error| error.to_string())?;
    assert!(entry.path_bytes().contains(&b'\\'));
    assert_rejected(bytes, &["src/lib.rs"], "invalid path")
}

#[test]
fn links_and_compressed_limit_are_rejected() -> Result<(), String> {
    let fixture = FixtureBuilder::root()?;
    let path = fixture.path().join("egui-0.36.1.crate");
    let mut tar = Builder::new(GzEncoder::new(Vec::new(), Compression::default()));
    let mut header = Header::new_gnu();
    header.set_entry_type(EntryType::Symlink);
    header.set_size(0);
    header
        .set_link_name("outside")
        .map_err(|error| error.to_string())?;
    header.set_cksum();
    tar.append_data(&mut header, "egui-0.36.1/src/lib.rs", Cursor::new([]))
        .map_err(|error| error.to_string())?;
    let bytes = tar
        .into_inner()
        .map_err(|error| error.to_string())?
        .finish()
        .map_err(|error| error.to_string())?;
    fs::write(&path, &bytes).map_err(|error| error.to_string())?;
    assert!(rust_file_hashes(&path, &lock(&bytes), &manifest(&["src/lib.rs"])).is_err());
    let oversized = fixture.path().join("oversized.crate");
    fs::File::create(&oversized)
        .map_err(|error| error.to_string())?
        .set_len(64 * 1024 * 1024 + 1)
        .map_err(|error| error.to_string())?;
    let error = rust_file_hashes(&oversized, &lock(b"x"), &manifest(&[]))
        .err()
        .ok_or_else(|| "oversized archive unexpectedly passed validation".to_owned())?;
    assert!(error.contains("64MiB"));
    Ok(())
}

#[path = "archive_security_tests.rs"]
mod security_tests;
