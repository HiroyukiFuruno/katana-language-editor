use std::fs;
use std::path::{Component, Path, PathBuf};

pub(super) fn safe_root(path: &Path) -> Result<PathBuf, String> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| format!("artifact root is missing: {error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("artifact root must be a regular directory".into());
    }
    path.canonicalize()
        .map_err(|error| format!("artifact root cannot be canonicalized: {error}"))
}

pub(super) fn safe_dir(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let path = safe_path(root, relative, "consumer profile directory")?;
    let metadata = fs::symlink_metadata(&path)
        .map_err(|error| format!("consumer profile directory is missing: {error}"))?;
    if !metadata.is_dir() {
        return Err(format!(
            "consumer profile path is not a directory: {relative}"
        ));
    }
    Ok(path)
}

pub(super) fn safe_file(root: &Path, relative: &str, kind: &str) -> Result<PathBuf, String> {
    let path = safe_path(root, relative, kind)?;
    let metadata =
        fs::symlink_metadata(&path).map_err(|error| format!("{kind} is missing: {error}"))?;
    if !metadata.is_file() {
        return Err(format!("{kind} is not a regular file: {relative}"));
    }
    Ok(path)
}

fn safe_path(root: &Path, relative: &str, kind: &str) -> Result<PathBuf, String> {
    if relative.trim().is_empty() || relative.contains('\\') || Path::new(relative).is_absolute() {
        return Err(format!("{kind} path is not safe: {relative:?}"));
    }
    let mut current = root.to_path_buf();
    for component in Path::new(relative).components() {
        let Component::Normal(part) = component else {
            return Err(format!("{kind} path is not safe: {relative:?}"));
        };
        current.push(part);
        let metadata = fs::symlink_metadata(&current)
            .map_err(|error| format!("{kind} path is missing: {error}"))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("{kind} path contains a symlink: {relative}"));
        }
    }
    let canonical = current
        .canonicalize()
        .map_err(|error| format!("{kind} path cannot be canonicalized: {error}"))?;
    if !canonical.starts_with(root) {
        return Err(format!("{kind} path escapes the artifact root: {relative}"));
    }
    Ok(canonical)
}
