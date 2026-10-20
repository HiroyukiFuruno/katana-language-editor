use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn checksum_manifest(root: &Path) -> Result<PathBuf, String> {
    let canonical_root = canonical_root(root)?;
    let path = root.join(".cargo-checksum.json");
    validate_regular_file(
        &canonical_root,
        &path,
        Path::new(".cargo-checksum.json"),
        "checksum manifest",
    )
}

pub(super) fn source_path(root: &Path, relative: &str) -> Result<PathBuf, String> {
    validate_relative_path(relative)?;
    let canonical_root = canonical_root(root)?;
    let path = root.join(relative);
    validate_regular_file(
        &canonical_root,
        &path,
        Path::new(relative),
        &format!("source path {relative:?}"),
    )
}

fn canonical_root(root: &Path) -> Result<PathBuf, String> {
    let metadata = fs::symlink_metadata(root)
        .map_err(|error| format!("external source root is unavailable: {error}"))?;
    if !metadata.is_dir() {
        return Err(format!(
            "external source root is not a directory: {}",
            root.display()
        ));
    }
    root.canonicalize()
        .map_err(|error| format!("external source root cannot be canonicalized: {error}"))
}

fn validate_regular_file(
    root: &Path,
    path: &Path,
    relative: &Path,
    label: &str,
) -> Result<PathBuf, String> {
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(component.as_os_str());
        let metadata = fs::symlink_metadata(&current)
            .map_err(|error| format!("{label} is unavailable: {}: {error}", current.display()))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("{label} contains a symlink: {}", current.display()));
        }
    }
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("{label} cannot be canonicalized: {error}"))?;
    if !canonical.starts_with(root) {
        return Err(format!(
            "{label} escapes the external source root: {}",
            path.display()
        ));
    }
    let metadata =
        fs::metadata(path).map_err(|error| format!("{label} is unavailable: {error}"))?;
    if !metadata.is_file() {
        return Err(format!("{label} is not a regular file: {}", path.display()));
    }
    Ok(canonical)
}

fn validate_relative_path(relative: &str) -> Result<(), String> {
    if relative.is_empty() {
        return Err("external source path is empty".into());
    }
    if relative.starts_with('/') || relative.contains('\\') || has_windows_prefix(relative) {
        return Err(format!(
            "external source path is not a relative POSIX path: {relative:?}"
        ));
    }
    if relative
        .split('/')
        .any(|component| component.is_empty() || component == "." || component == "..")
    {
        return Err(format!(
            "external source path has an invalid component: {relative:?}"
        ));
    }
    Ok(())
}

fn has_windows_prefix(path: &str) -> bool {
    path.as_bytes()
        .get(1)
        .is_some_and(|colon| *colon == b':' && path.as_bytes()[0].is_ascii_alphabetic())
}
