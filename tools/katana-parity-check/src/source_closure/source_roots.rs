use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::fingerprint::sha256_hex;
use super::operational_evidence::{validate_relative_path, validate_rust_source_path};
use super::operational_input::FIXED_KATANA_REVISION;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceRootManifest {
    schema_version: String,
    katana_revision: String,
    source_universe_sha256: String,
    directory_roots: Vec<String>,
    file_roots: Vec<String>,
}

pub(super) fn resolve(
    manifest_path: &Path,
    source_universe_path: &Path,
    katana_root: &Path,
    source_universe_fingerprint: &str,
) -> Result<Vec<String>, String> {
    let bytes = std::fs::read(manifest_path)
        .map_err(|error| format!("failed to read source-root manifest: {error}"))?;
    let manifest: SourceRootManifest = serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid source-root manifest: {error}"))?;
    let source_universe = std::fs::read(source_universe_path)
        .map_err(|error| format!("failed to read source-universe input: {error}"))?;
    validate_source_universe_document(&source_universe)?;
    validate_manifest(&manifest, &source_universe, source_universe_fingerprint)?;
    super::source_roots_ledger::validate_documented_directories(
        &source_universe,
        &manifest.directory_roots,
    )?;
    let mut paths = BTreeSet::new();
    for root in &manifest.directory_roots {
        collect_directory_sources(katana_root, root, &mut paths)?;
    }
    for root in &manifest.file_roots {
        validate_rust_source_path(root)?;
        validate_file_root(katana_root, root)?;
        paths.insert(root.clone());
    }
    super::source_roots_ledger::validate_documented_file_roots(
        &source_universe,
        &manifest.file_roots,
    )?;
    if paths.is_empty() {
        return Err("source-root manifest resolves to no Rust source files".into());
    }
    super::source_roots_ledger::validate_documented_paths(&source_universe, katana_root, &paths)?;
    Ok(paths.into_iter().collect())
}

fn validate_manifest(
    manifest: &SourceRootManifest,
    source_universe: &[u8],
    source_universe_fingerprint: &str,
) -> Result<(), String> {
    if manifest.schema_version != "1" {
        return Err("source-root manifest schema_version must be exactly \"1\"".into());
    }
    if manifest.katana_revision != FIXED_KATANA_REVISION {
        return Err("source-root manifest does not use the fixed KatanA revision".into());
    }
    let source_universe_sha256 = sha256_hex(source_universe);
    if manifest.source_universe_sha256 != source_universe_sha256
        || source_universe_fingerprint != source_universe_sha256
    {
        return Err("source-root manifest does not match captured source-universe bytes".into());
    }
    if manifest.directory_roots.is_empty() || manifest.file_roots.is_empty() {
        return Err("source-root manifest requires directory_roots and file_roots".into());
    }
    for root in manifest.directory_roots.iter().chain(&manifest.file_roots) {
        validate_relative_path(root)?;
    }
    Ok(())
}

pub(super) fn validate_source_universe_document(bytes: &[u8]) -> Result<(), String> {
    let document = std::str::from_utf8(bytes)
        .map_err(|error| format!("source-universe input is not UTF-8: {error}"))?;
    if !document.starts_with("# KatanA Editor Source Universe\n")
        || !document.contains(FIXED_KATANA_REVISION)
    {
        return Err("source-universe input does not identify the fixed KatanA audit".into());
    }
    Ok(())
}

fn collect_directory_sources(
    katana_root: &Path,
    relative: &str,
    paths: &mut BTreeSet<String>,
) -> Result<(), String> {
    let directory = canonical_child(katana_root, relative, "source-root directory")?;
    if !directory.is_dir() {
        return Err(format!(
            "source-root directory is not a directory: {relative}"
        ));
    }
    collect_directory_sources_recursive(katana_root, &directory, paths)
}

fn collect_directory_sources_recursive(
    katana_root: &Path,
    directory: &Path,
    paths: &mut BTreeSet<String>,
) -> Result<(), String> {
    let entries = std::fs::read_dir(directory)
        .map_err(|error| format!("failed to read source-root directory: {error}"))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("failed to read source-root entry: {error}"))?;
        let path = entry.path();
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|error| format!("failed to inspect source-root entry: {error}"))?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "source-root entry must not be a symlink: {}",
                path.display()
            ));
        }
        if metadata.is_dir() {
            collect_directory_sources_recursive(katana_root, &path, paths)?;
        } else if metadata.is_file() && path.extension().is_some_and(|extension| extension == "rs")
        {
            let relative = path
                .strip_prefix(katana_root)
                .map_err(|_| "source-root file escapes KatanA checkout".to_string())?;
            let relative = relative.to_string_lossy().replace('\\', "/");
            validate_rust_source_path(&relative)?;
            paths.insert(relative);
        }
    }
    Ok(())
}

fn validate_file_root(katana_root: &Path, relative: &str) -> Result<(), String> {
    let path = canonical_child(katana_root, relative, "source-root file")?;
    if !path.is_file() {
        return Err(format!("source-root file is not a file: {relative}"));
    }
    Ok(())
}

fn canonical_child(root: &Path, relative: &str, label: &str) -> Result<PathBuf, String> {
    let path = root
        .join(relative)
        .canonicalize()
        .map_err(|error| format!("{label} is missing or unreadable: {relative}: {error}"))?;
    if !path.starts_with(root) {
        return Err(format!("{label} escapes KatanA checkout: {relative}"));
    }
    Ok(path)
}
