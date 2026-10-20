use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::source_roots;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn manifest_expands_only_declared_directories_and_files() -> TestResult {
    let root = fixture_root("expands")?;
    write(&root, "direct/nested/direct.rs", "fn direct() {}")?;
    write(&root, "direct/other.txt", "not Rust")?;
    write(&root, "mixed/included.rs", "fn included() {}")?;
    write(&root, "mixed/unrelated.rs", "fn unrelated() {}")?;
    let manifest = root.join("roots.json");
    let source_universe = root.join("source-universe.md");
    let source_universe_bytes = source_universe_bytes();
    fs::write(&source_universe, &source_universe_bytes)?;
    fs::write(
        &manifest,
        format!(
            r#"{{
  "schema_version": "1",
  "katana_revision": "{}",
  "source_universe_sha256": "{}",
  "directory_roots": ["direct"],
  "file_roots": ["mixed/included.rs"]
}}"#,
            super::operational_input::FIXED_KATANA_REVISION,
            super::fingerprint::sha256_hex(&source_universe_bytes)
        ),
    )?;

    let resolved = source_roots::resolve(
        &manifest,
        &source_universe,
        &root,
        &super::fingerprint::sha256_hex(&source_universe_bytes),
    )?;
    assert_eq!(
        resolved,
        vec![
            "direct/nested/direct.rs".to_string(),
            "mixed/included.rs".to_string()
        ]
    );
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn manifest_rejects_a_missing_declared_file_root() -> TestResult {
    let root = fixture_root("missing-file")?;
    write(&root, "direct/root.rs", "fn root() {}")?;
    let manifest = root.join("roots.json");
    let source_universe = root.join("source-universe.md");
    let source_universe_bytes = source_universe_bytes();
    fs::write(&source_universe, &source_universe_bytes)?;
    fs::write(
        &manifest,
        format!(
            r#"{{
  "schema_version": "1",
  "katana_revision": "{}",
  "source_universe_sha256": "{}",
  "directory_roots": ["direct"],
  "file_roots": ["mixed/missing.rs"]
}}"#,
            super::operational_input::FIXED_KATANA_REVISION,
            super::fingerprint::sha256_hex(&source_universe_bytes)
        ),
    )?;

    let rejected = matches!(
        source_roots::resolve(
            &manifest,
            &source_universe,
            &root,
            &super::fingerprint::sha256_hex(&source_universe_bytes),
        ),
        Err(error) if error.contains("source-root file is missing or unreadable")
    );
    let _ = fs::remove_dir_all(root);
    assert!(rejected);
    Ok(())
}

#[test]
fn manifest_rejects_source_universe_fingerprint_drift() -> TestResult {
    let root = fixture_root("source-universe-drift")?;
    let manifest = root.join("roots.json");
    let source_universe = root.join("source-universe.md");
    let source_universe_bytes = source_universe_bytes();
    fs::write(&source_universe, &source_universe_bytes)?;
    fs::write(
        &manifest,
        format!(
            r#"{{
  "schema_version": "1",
  "katana_revision": "{}",
  "source_universe_sha256": "{}",
  "directory_roots": ["direct"],
  "file_roots": ["mixed/included.rs"]
}}"#,
            super::operational_input::FIXED_KATANA_REVISION,
            "0".repeat(64)
        ),
    )?;

    let rejected = matches!(
        source_roots::resolve(
            &manifest,
            &source_universe,
            &root,
            &super::fingerprint::sha256_hex(&source_universe_bytes),
        ),
        Err(error) if error.contains("does not match captured source-universe bytes")
    );
    let _ = fs::remove_dir_all(root);
    assert!(rejected);
    Ok(())
}

#[test]
fn manifest_rejects_a_documented_source_path_omission() -> TestResult {
    let root = fixture_root("documented-omission")?;
    write(&root, "crates/katana-ui/src/direct/root.rs", "fn root() {}")?;
    write(
        &root,
        "crates/katana-ui/src/mixed/included.rs",
        "fn included() {}",
    )?;
    write(
        &root,
        "crates/katana-ui/src/views/documented.rs",
        "fn documented() {}",
    )?;
    let manifest = root.join("roots.json");
    let source_universe = root.join("source-universe.md");
    let source_universe_bytes = format!(
        "# KatanA Editor Source Universe\n\n{}\n\ncrates/katana-ui/src/mixed/included.rs\nviews/documented.rs\n",
        super::operational_input::FIXED_KATANA_REVISION
    )
    .into_bytes();
    fs::write(&source_universe, &source_universe_bytes)?;
    fs::write(
        &manifest,
        format!(
            r#"{{
  "schema_version": "1",
  "katana_revision": "{}",
  "source_universe_sha256": "{}",
  "directory_roots": ["crates/katana-ui/src/direct"],
  "file_roots": ["crates/katana-ui/src/mixed/included.rs"]
}}"#,
            super::operational_input::FIXED_KATANA_REVISION,
            super::fingerprint::sha256_hex(&source_universe_bytes)
        ),
    )?;

    let rejected = matches!(
        source_roots::resolve(
            &manifest,
            &source_universe,
            &root,
            &super::fingerprint::sha256_hex(&source_universe_bytes),
        ),
        Err(error) if error.contains("omits documented source path")
    );
    let _ = fs::remove_dir_all(root);
    assert!(rejected);
    Ok(())
}

fn fixture_root(name: &str) -> TestResult<PathBuf> {
    let suffix = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let root = std::env::temp_dir().join(format!(
        "katana-parity-source-roots-{name}-{}-{suffix}",
        std::process::id()
    ));
    fs::create_dir_all(&root)?;
    Ok(root.canonicalize()?)
}

fn write(root: &std::path::Path, relative: &str, contents: &str) -> TestResult {
    let path = root.join(relative);
    let parent = path.parent().ok_or("fixture path has no parent")?;
    fs::create_dir_all(parent)?;
    fs::write(path, contents)?;
    Ok(())
}

fn source_universe_bytes() -> Vec<u8> {
    format!(
        "# KatanA Editor Source Universe\n\n{}\n\nmixed/included.rs\n",
        super::operational_input::FIXED_KATANA_REVISION
    )
    .into_bytes()
}
