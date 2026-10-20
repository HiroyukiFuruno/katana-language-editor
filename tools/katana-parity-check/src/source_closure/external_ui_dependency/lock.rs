use std::fs;
use std::path::{Component, Path, PathBuf};

use serde_json::Value;

use super::super::fingerprint::sha256_hex;
use super::PACKAGE;
use crate::system::ProcessService;

#[derive(Clone, Debug, Default)]
pub(super) struct LockPackage {
    pub(super) version: String,
    pub(super) source: String,
    pub(super) checksum: String,
    pub(super) lock_sha256: String,
}

pub(super) fn parse_lock(bytes: &[u8], unresolved: &mut Vec<String>) -> Option<LockPackage> {
    let value: Value = match toml::from_slice(bytes) {
        Ok(value) => value,
        Err(error) => {
            unresolved.push(format!("fixed-reference Cargo.lock parse failure: {error}"));
            return None;
        }
    };
    let packages = value.get("package").and_then(Value::as_array);
    let matches = packages
        .into_iter()
        .flatten()
        .filter(|package| package.get("name").and_then(Value::as_str) == Some(PACKAGE))
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        unresolved.push(format!(
            "egui package is {} in fixed-reference Cargo.lock; exactly one is required",
            if matches.is_empty() {
                "missing".into()
            } else {
                matches.len().to_string()
            }
        ));
        return None;
    }
    let package = matches[0];
    let version = package
        .get("version")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let source = package
        .get("source")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let checksum = package
        .get("checksum")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if version.is_empty() || source.is_empty() || checksum.is_empty() {
        unresolved.push("egui lock package lacks fixed version/source/checksum evidence".into());
    }
    Some(LockPackage {
        version: version.into(),
        source: source.into(),
        checksum: checksum.into(),
        lock_sha256: sha256_hex(bytes),
    })
}

pub(super) fn resolve_package_root(
    reference_root: &Path,
    lock: &LockPackage,
    unresolved: &mut Vec<String>,
) -> Option<PathBuf> {
    let output = ProcessService::create_command("cargo")
        .args(["metadata", "--locked", "--offline", "--format-version", "1"])
        .arg("--manifest-path")
        .arg(reference_root.join("Cargo.toml"))
        .output();
    let output = match output {
        Ok(output) if output.status.success() => output,
        Ok(output) => {
            unresolved.push(format!(
                "cargo metadata could not resolve fixed egui source: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
            return None;
        }
        Err(error) => {
            unresolved.push(format!(
                "cargo metadata could not run for fixed reference: {error}"
            ));
            return None;
        }
    };
    let metadata: Value = match serde_json::from_slice(&output.stdout) {
        Ok(value) => value,
        Err(error) => {
            unresolved.push(format!("cargo metadata JSON parse failure: {error}"));
            return None;
        }
    };
    let packages = metadata
        .get("packages")
        .and_then(Value::as_array)
        .into_iter()
        .flatten();
    let matches = packages
        .filter(|package| {
            package.get("name").and_then(Value::as_str) == Some(PACKAGE)
                && package.get("version").and_then(Value::as_str) == Some(lock.version.as_str())
                && package.get("source").and_then(Value::as_str) == Some(lock.source.as_str())
        })
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        unresolved.push(format!(
            "fixed cargo metadata resolves {} matching egui packages",
            matches.len()
        ));
        return None;
    }
    let manifest = matches[0].get("manifest_path").and_then(Value::as_str)?;
    let root = Path::new(manifest).parent()?.to_path_buf();
    if !root.is_dir() {
        unresolved.push(format!(
            "resolved egui source directory is unavailable: {}",
            root.display()
        ));
        None
    } else {
        Some(root)
    }
}

pub(super) fn resolve_package_archive(
    root: &Path,
    lock: &LockPackage,
    unresolved: &mut Vec<String>,
) -> Option<PathBuf> {
    let expected = format!("{PACKAGE}-{}", lock.version);
    if root.file_name().and_then(|name| name.to_str()) != Some(expected.as_str()) {
        unresolved
            .push("resolved egui source root does not match its fixed package version".into());
        return None;
    }
    let Some(registry) = root.parent() else {
        unresolved.push("resolved egui source root has an unknown registry layout".into());
        return None;
    };
    let Some(src) = registry.parent() else {
        unresolved.push("resolved egui source root has an unknown registry layout".into());
        return None;
    };
    let Some(registry_name) = registry.file_name() else {
        unresolved.push("resolved egui source root has an unknown registry layout".into());
        return None;
    };
    if src.file_name().and_then(|name| name.to_str()) != Some("src")
        || registry_name.to_str().is_none()
        || root
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        unresolved.push("resolved egui source root has an unknown registry layout".into());
        return None;
    }
    let archive = src.parent().map(|parent| {
        parent
            .join("cache")
            .join(registry_name)
            .join(format!("{expected}.crate"))
    })?;
    match fs::symlink_metadata(&archive) {
        Ok(metadata) if metadata.file_type().is_file() && !metadata.file_type().is_symlink() => {
            Some(archive)
        }
        Ok(_) => {
            unresolved.push(format!(
                "egui registry archive is not a regular file: {}",
                archive.display()
            ));
            None
        }
        Err(error) => {
            unresolved.push(format!(
                "egui registry archive is unavailable: {}: {error}",
                archive.display()
            ));
            None
        }
    }
}
