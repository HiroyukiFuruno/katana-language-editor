use std::fs;
use std::path::Path;

use super::fingerprint::sha256_hex;

pub(super) fn evidence_tree_fingerprint_for(root: &Path) -> Result<String, String> {
    let mut files = Vec::new();
    collect(root, root, &mut files)?;
    fingerprint_files(root, &mut files, Some("evidence/"))
}

pub(super) fn evidence_tree_fingerprint_for_staging(root: &Path) -> Result<String, String> {
    let mut files = Vec::new();
    for name in ["profiles", "provenance"] {
        collect(root, &root.join(name), &mut files)?;
    }
    fingerprint_files(root, &mut files, Some("evidence/"))
}

fn fingerprint_files(
    root: &Path,
    files: &mut [String],
    prefix: Option<&str>,
) -> Result<String, String> {
    if let Some(prefix) = prefix {
        files
            .iter_mut()
            .for_each(|path| *path = format!("{prefix}{path}"));
    }
    files.sort();
    let mut bytes = Vec::new();
    for path in files {
        let source = path.strip_prefix("evidence/").unwrap_or(path);
        bytes.extend_from_slice(path.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(&fs::read(root.join(source)).map_err(|error| {
            format!(
                "evidence is unreadable {}: {error}",
                root.join(source).display()
            )
        })?);
        bytes.push(0);
    }
    Ok(sha256_hex(&bytes))
}

fn collect(root: &Path, current: &Path, files: &mut Vec<String>) -> Result<(), String> {
    for entry in
        fs::read_dir(current).map_err(|error| format!("evidence is unreadable: {error}"))?
    {
        let entry = entry.map_err(|error| format!("evidence is unreadable: {error}"))?;
        let path = entry.path();
        let meta = fs::symlink_metadata(&path)
            .map_err(|error| format!("evidence is unreadable: {error}"))?;
        if meta.file_type().is_symlink() {
            return Err("evidence contains a symlink".into());
        }
        if meta.is_dir() {
            collect(root, &path, files)?;
        } else if meta.is_file() {
            files.push(
                path.strip_prefix(root)
                    .map_err(|_| "evidence escaped root".to_string())?
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        } else {
            return Err("evidence contains a special file".into());
        }
    }
    Ok(())
}
