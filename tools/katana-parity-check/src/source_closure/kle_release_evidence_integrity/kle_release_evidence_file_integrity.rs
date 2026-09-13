use super::super::fingerprint::sha256_hex;
use super::super::kle_release_evidence::{BeforeAfterFiles, EvidenceFile};
use std::path::{Component, Path, PathBuf};

pub(super) fn validate_file_pair(
    root: &Path,
    files: &BeforeAfterFiles,
    kind: &str,
) -> Result<(), String> {
    validate_file(root, &files.before, kind)?;
    validate_file(root, &files.after, kind)
}
fn validate_file(root: &Path, file: &EvidenceFile, kind: &str) -> Result<(), String> {
    super::sha256("evidence file sha256", &file.sha256)?;
    let path = safe_file(root, &file.relative_path, kind)?;
    let bytes = std::fs::read(path)
        .map_err(|error| format!("{kind} evidence file cannot be read: {error}"))?;
    if bytes.is_empty() || sha256_hex(&bytes) != file.sha256 {
        return Err(format!("{kind} evidence file hash does not match"));
    }
    Ok(())
}
fn safe_file(root: &Path, value: &str, kind: &str) -> Result<PathBuf, String> {
    if value.is_empty()
        || value.contains('\\')
        || has_drive_prefix(value)
        || Path::new(value).is_absolute()
    {
        return Err(format!("{kind} evidence path is not a safe relative path"));
    }
    let mut current = root.to_path_buf();
    for component in Path::new(value).components() {
        let Component::Normal(part) = component else {
            return Err(format!("{kind} evidence path is not a safe relative path"));
        };
        current.push(part);
        let metadata = std::fs::symlink_metadata(&current)
            .map_err(|error| format!("{kind} evidence path is missing: {error}"))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("{kind} evidence path contains a symlink"));
        }
    }
    if !std::fs::symlink_metadata(&current)
        .map_err(|error| error.to_string())?
        .is_file()
    {
        return Err(format!("{kind} evidence file must be a regular file"));
    }
    let canonical = current
        .canonicalize()
        .map_err(|error| format!("{kind} evidence file cannot be canonicalized: {error}"))?;
    if !canonical.starts_with(root) {
        return Err(format!("{kind} evidence file escapes the artifact root"));
    }
    Ok(canonical)
}
fn has_drive_prefix(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}
