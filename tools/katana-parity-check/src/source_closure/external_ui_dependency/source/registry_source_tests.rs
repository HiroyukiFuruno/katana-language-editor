use std::fs;
use std::io::Cursor;

use flate2::{Compression, write::GzEncoder};
use tar::{Builder, Header};

use super::super::super::super::capability_manifest::capability_manifest_test_fixtures::FixtureBuilder;
use super::super::super::fingerprint::sha256_hex;
use super::super::lock::LockPackage;
use super::dependency_sources;

const SOURCE: &str = "struct TextEdit; impl TextEdit { fn multiline() {} fn load_state() {} fn store_state() {} fn show() {} } enum Event { Paste } struct InputState; impl InputState { fn consume_shortcut() {} }";
const REGULAR_FILE_MODE: u32 = 0o644;

fn write_archive(
    root: &std::path::Path,
    paths: &[&str],
) -> Result<(std::path::PathBuf, String), String> {
    let mut tar = Builder::new(GzEncoder::new(Vec::new(), Compression::default()));
    for path in paths {
        let mut header = Header::new_gnu();
        header.set_size(SOURCE.len() as u64);
        header.set_mode(REGULAR_FILE_MODE);
        header.set_cksum();
        tar.append_data(
            &mut header,
            format!("egui-0.36.1/{path}"),
            Cursor::new(SOURCE.as_bytes()),
        )
        .map_err(|error| error.to_string())?;
    }
    let bytes = tar
        .into_inner()
        .map_err(|error| error.to_string())?
        .finish()
        .map_err(|error| error.to_string())?;
    let archive = root.join("egui-0.36.1.crate");
    fs::write(&archive, &bytes).map_err(|error| error.to_string())?;
    Ok((
        archive,
        sha256_hex(&bytes).trim_start_matches("sha256:").to_owned(),
    ))
}

fn lock(checksum: String) -> LockPackage {
    LockPackage {
        version: "0.36.1".into(),
        checksum,
        ..LockPackage::default()
    }
}

#[test]
fn registry_source_without_checksum_manifest_is_authenticated_and_scanned() -> Result<(), String> {
    let fixture = FixtureBuilder::root()?;
    fs::create_dir(fixture.path().join("src")).map_err(|error| error.to_string())?;
    fs::write(fixture.path().join("src/editor.rs"), SOURCE).map_err(|error| error.to_string())?;
    let (archive, checksum) = write_archive(fixture.path(), &["src/editor.rs"])?;
    let (files, symbols, spans, _, unresolved) =
        dependency_sources(fixture.path(), &archive, &lock(checksum));
    assert_eq!(files.len(), 1);
    assert_eq!(symbols.len(), 6);
    assert!(!spans.is_empty());
    assert!(unresolved.is_empty(), "{unresolved:?}");
    Ok(())
}

#[test]
fn source_rust_paths_must_match_the_authenticated_archive() -> Result<(), String> {
    let fixture = FixtureBuilder::root()?;
    fs::create_dir(fixture.path().join("src")).map_err(|error| error.to_string())?;
    fs::write(fixture.path().join("src/editor.rs"), SOURCE).map_err(|error| error.to_string())?;
    let (archive, checksum) = write_archive(fixture.path(), &["src/editor.rs", "src/missing.rs"])?;
    let (_, _, _, _, unresolved) = dependency_sources(fixture.path(), &archive, &lock(checksum));
    assert!(
        unresolved
            .iter()
            .any(|item| item.contains("source Rust paths differ from registry archive"))
    );
    fs::write(fixture.path().join("src/added.rs"), SOURCE).map_err(|error| error.to_string())?;
    let (archive, checksum) = write_archive(fixture.path(), &["src/editor.rs"])?;
    let (_, _, _, _, unresolved) = dependency_sources(fixture.path(), &archive, &lock(checksum));
    assert!(
        unresolved
            .iter()
            .any(|item| item.contains("source Rust paths differ from registry archive"))
    );
    Ok(())
}

#[test]
fn registry_source_tampering_produces_no_partial_ast_evidence() -> Result<(), String> {
    let fixture = FixtureBuilder::root()?;
    fs::create_dir(fixture.path().join("src")).map_err(|error| error.to_string())?;
    fs::write(fixture.path().join("src/editor.rs"), SOURCE).map_err(|error| error.to_string())?;
    fs::write(fixture.path().join("src/other.rs"), "struct Tampered;")
        .map_err(|error| error.to_string())?;
    let (archive, checksum) = write_archive(fixture.path(), &["src/editor.rs", "src/other.rs"])?;
    let (files, symbols, spans, _, unresolved) =
        dependency_sources(fixture.path(), &archive, &lock(checksum));
    assert!(files.is_empty() && symbols.is_empty() && spans.is_empty());
    assert!(
        unresolved
            .iter()
            .any(|item| item.contains("source/hash mismatch: src/other.rs")),
        "{unresolved:?}"
    );
    Ok(())
}

#[test]
fn present_but_invalid_checksum_manifest_is_not_ignored() -> Result<(), String> {
    let fixture = FixtureBuilder::root()?;
    fs::create_dir(fixture.path().join("src")).map_err(|error| error.to_string())?;
    fs::write(fixture.path().join("src/editor.rs"), SOURCE).map_err(|error| error.to_string())?;
    let (archive, checksum) = write_archive(fixture.path(), &["src/editor.rs"])?;
    let missing_files = serde_json::json!({"package": checksum}).to_string();
    let invalid_hash = serde_json::json!({
        "package": checksum, "files": {"src/editor.rs": "invalid"}
    })
    .to_string();
    for (manifest, reason) in [
        ("not JSON", "manifest is malformed"),
        ("{}", "differs from Cargo.lock checksum"),
        (missing_files.as_str(), "has no files object"),
        (invalid_hash.as_str(), "hash is invalid"),
    ] {
        fs::write(fixture.path().join(".cargo-checksum.json"), manifest)
            .map_err(|error| error.to_string())?;
        let (files, symbols, spans, _, unresolved) =
            dependency_sources(fixture.path(), &archive, &lock(checksum.clone()));
        assert!(files.is_empty() && symbols.is_empty() && spans.is_empty());
        assert!(
            unresolved.iter().any(|item| item.contains(reason)),
            "{unresolved:?}"
        );
    }
    Ok(())
}

#[cfg(unix)]
#[test]
fn registry_source_without_manifest_rejects_symlinked_rust_source() -> Result<(), String> {
    let fixture = FixtureBuilder::root()?;
    let outside = FixtureBuilder::root()?;
    fs::create_dir(fixture.path().join("src")).map_err(|error| error.to_string())?;
    fs::write(outside.path().join("editor.rs"), SOURCE).map_err(|error| error.to_string())?;
    std::os::unix::fs::symlink(
        outside.path().join("editor.rs"),
        fixture.path().join("src/editor.rs"),
    )
    .map_err(|error| error.to_string())?;
    let (archive, checksum) = write_archive(fixture.path(), &["src/editor.rs"])?;
    let (files, symbols, spans, _, unresolved) =
        dependency_sources(fixture.path(), &archive, &lock(checksum));
    assert!(files.is_empty() && symbols.is_empty() && spans.is_empty());
    assert!(
        unresolved
            .iter()
            .any(|item| item.contains("contains a symlink")),
        "{unresolved:?}"
    );
    Ok(())
}
