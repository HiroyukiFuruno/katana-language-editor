use std::fs;
use std::io::Cursor;

use flate2::{Compression, write::GzEncoder};
use serde_json::{Map, json};
use tar::{Builder, Header};

use super::super::super::super::capability_manifest::capability_manifest_test_fixtures::FixtureBuilder;
use super::super::super::fingerprint::sha256_hex;
use super::super::lock::LockPackage;
use super::{dependency_sources, path_validation};

const REGULAR_FILE_MODE: u32 = 0o644;

fn lock(checksum: String) -> LockPackage {
    LockPackage {
        version: "0.36.1".into(),
        checksum,
        ..LockPackage::default()
    }
}

fn valid_source() -> &'static str {
    "struct TextEdit; impl TextEdit { fn multiline() {} fn load_state() {} fn store_state() {} fn show() {} } enum Event { Paste } struct InputState; impl InputState { fn consume_shortcut() {} }"
}

fn hash(bytes: &[u8]) -> String {
    sha256_hex(bytes).trim_start_matches("sha256:").to_owned()
}

fn write_manifest(
    root: &std::path::Path,
    checksum: &str,
    files: &[(&str, &str)],
) -> Result<(), String> {
    let files = files
        .iter()
        .map(|(path, value)| ((*path).to_owned(), json!(value)))
        .collect::<Map<_, _>>();
    fs::write(
        root.join(".cargo-checksum.json"),
        serde_json::to_vec(&json!({"package": checksum, "files": files}))
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())
}

fn write_archive(
    root: &std::path::Path,
    entries: &[(&str, &str)],
) -> Result<(std::path::PathBuf, String), String> {
    let mut tar = Builder::new(GzEncoder::new(Vec::new(), Compression::default()));
    for (path, contents) in entries {
        let mut header = Header::new_gnu();
        header.set_size(contents.len() as u64);
        header.set_mode(REGULAR_FILE_MODE);
        header.set_cksum();
        tar.append_data(
            &mut header,
            format!("egui-0.36.1/{path}"),
            Cursor::new(contents.as_bytes()),
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
    Ok((archive, hash(&bytes)))
}

fn assert_rejected(unresolved: &[String], reason: &str) {
    assert!(
        unresolved.iter().any(|item| item.contains(reason)),
        "missing {reason:?}: {unresolved:?}"
    );
}

#[test]
fn normal_source_is_read_and_scanned() -> Result<(), String> {
    let fixture = FixtureBuilder::root()?;
    fs::create_dir(fixture.path().join("src")).map_err(|error| error.to_string())?;
    fs::write(fixture.path().join("src/editor.rs"), valid_source())
        .map_err(|error| error.to_string())?;
    let (archive, checksum) = write_archive(fixture.path(), &[("src/editor.rs", valid_source())])?;
    write_manifest(
        fixture.path(),
        &checksum,
        &[("src/editor.rs", &hash(valid_source().as_bytes()))],
    )?;
    let (files, symbols, _, _, unresolved) =
        dependency_sources(fixture.path(), &archive, &lock(checksum.clone()));
    assert_eq!(files.len(), 1);
    assert_eq!(symbols.len(), 6);
    assert!(unresolved.is_empty(), "{unresolved:?}");
    let replacement = "struct TextEdit; impl TextEdit { fn multiline() {} fn load_state() {} fn store_state() {} fn show() {} } enum Event { Paste } struct InputState; impl InputState { fn consume_shortcut() {} } const REPLACED: bool = true;";
    fs::write(fixture.path().join("src/editor.rs"), replacement)
        .map_err(|error| error.to_string())?;
    write_manifest(
        fixture.path(),
        &checksum,
        &[("src/editor.rs", &hash(replacement.as_bytes()))],
    )?;
    let (files, symbols, spans, _, unresolved) =
        dependency_sources(fixture.path(), &archive, &lock(checksum));
    assert!(files.is_empty() && symbols.is_empty() && spans.is_empty());
    assert_rejected(&unresolved, "archive files differ from checksum manifest");
    Ok(())
}

#[test]
fn invalid_path_forms_are_unresolved_without_fallback_reads() -> Result<(), String> {
    let fixture = FixtureBuilder::root()?;
    let package = fixture.path().join("package");
    fs::create_dir(&package).map_err(|error| error.to_string())?;
    fs::write(fixture.path().join("foreign.rs"), valid_source())
        .map_err(|error| error.to_string())?;
    for (path, reason) in [
        ("../foreign.rs", "invalid component"),
        ("/foreign.rs", "not a relative POSIX path"),
        ("src\\foreign.rs", "not a relative POSIX path"),
        ("C:foreign.rs", "not a relative POSIX path"),
        ("src/./foreign.rs", "invalid component"),
        ("src//foreign.rs", "invalid component"),
        ("", "path is empty"),
    ] {
        write_manifest(&package, "lock-checksum", &[(path, "not-used")])?;
        let (files, symbols, spans, _, unresolved) = dependency_sources(
            &package,
            &package.join("missing.crate"),
            &lock("lock-checksum".into()),
        );
        assert!(files.is_empty() && symbols.is_empty() && spans.is_empty());
        assert_rejected(&unresolved, reason);
    }
    Ok(())
}

#[test]
fn missing_or_non_file_checksum_sources_are_unresolved() -> Result<(), String> {
    let fixture = FixtureBuilder::root()?;
    let missing = fixture.path().join("missing.crate");
    let (_, symbols, _, _, unresolved) =
        dependency_sources(fixture.path(), &missing, &lock("lock-checksum".into()));
    assert!(symbols.is_empty());
    assert_rejected(&unresolved, "registry archive is unavailable");
    fs::create_dir(fixture.path().join("src")).map_err(|error| error.to_string())?;
    write_manifest(fixture.path(), "lock-checksum", &[("src", "not-used")])?;
    let (_, symbols, _, _, unresolved) =
        dependency_sources(fixture.path(), &missing, &lock("lock-checksum".into()));
    assert!(symbols.is_empty());
    assert_rejected(&unresolved, "not a regular file");
    write_manifest(
        fixture.path(),
        "lock-checksum",
        &[("src/missing.rs", "not-used")],
    )?;
    let (_, symbols, _, _, unresolved) =
        dependency_sources(fixture.path(), &missing, &lock("lock-checksum".into()));
    assert!(symbols.is_empty());
    assert_rejected(&unresolved, "is unavailable");
    Ok(())
}

#[cfg(unix)]
#[test]
fn source_and_checksum_symlinks_are_unresolved() -> Result<(), String> {
    use std::os::unix::fs::symlink;
    let fixture = FixtureBuilder::root()?;
    let outside = FixtureBuilder::root()?;
    fs::create_dir(fixture.path().join("src")).map_err(|error| error.to_string())?;
    fs::write(outside.path().join("foreign.rs"), valid_source())
        .map_err(|error| error.to_string())?;
    symlink(
        outside.path().join("foreign.rs"),
        fixture.path().join("src/file.rs"),
    )
    .map_err(|error| error.to_string())?;
    write_manifest(
        fixture.path(),
        "lock-checksum",
        &[("src/file.rs", &hash(valid_source().as_bytes()))],
    )?;
    let (_, symbols, _, _, unresolved) = dependency_sources(
        fixture.path(),
        &fixture.path().join("missing.crate"),
        &lock("lock-checksum".into()),
    );
    assert!(symbols.is_empty());
    assert_rejected(&unresolved, "contains a symlink");
    symlink(outside.path(), fixture.path().join("ancestor")).map_err(|error| error.to_string())?;
    write_manifest(
        fixture.path(),
        "lock-checksum",
        &[("ancestor/foreign.rs", &hash(valid_source().as_bytes()))],
    )?;
    let (files, symbols, spans, _, unresolved) = dependency_sources(
        fixture.path(),
        &fixture.path().join("missing.crate"),
        &lock("lock-checksum".into()),
    );
    assert!(files.is_empty() && symbols.is_empty() && spans.is_empty());
    assert_rejected(&unresolved, "contains a symlink");
    write_manifest(outside.path(), "lock-checksum", &[])?;
    fs::remove_file(fixture.path().join(".cargo-checksum.json"))
        .map_err(|error| error.to_string())?;
    symlink(
        outside.path().join(".cargo-checksum.json"),
        fixture.path().join(".cargo-checksum.json"),
    )
    .map_err(|error| error.to_string())?;
    let (_, symbols, _, _, unresolved) = dependency_sources(
        fixture.path(),
        &fixture.path().join("missing.crate"),
        &lock("lock-checksum".into()),
    );
    assert!(symbols.is_empty());
    assert_rejected(&unresolved, "checksum manifest contains a symlink");
    Ok(())
}

#[test]
fn path_validation_rejects_windows_prefix_without_platform_support() {
    assert!(path_validation::source_path(std::path::Path::new("."), "C:src.rs").is_err());
}
