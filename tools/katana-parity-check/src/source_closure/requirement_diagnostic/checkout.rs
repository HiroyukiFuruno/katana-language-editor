use std::path::{Component, Path, PathBuf};

use super::super::fingerprint::sha256_hex;
use super::super::operational_input::FIXED_KATANA_REVISION;
use super::super::operational_process::git_output;

pub(super) fn read_and_verify(root: &Path, relative: &str) -> Result<Vec<u8>, String> {
    let path = checked_path(root, &root.join(relative))?;
    let bytes = std::fs::read(&path)
        .map_err(|error| format!("failed to read source {relative}: {error}"))?;
    let blob = git_output(
        root,
        &[
            "cat-file",
            "blob",
            &format!("{FIXED_KATANA_REVISION}:{relative}"),
        ],
    )?;
    if bytes != blob {
        return Err(format!(
            "source mutation or fixed blob mismatch: {relative}"
        ));
    }
    Ok(bytes)
}

pub(super) fn verify_fixed_blob(
    root: &Path,
    relative: &str,
    scanned: Option<&str>,
) -> Result<(), String> {
    let bytes = read_and_verify(root, relative)?;
    if scanned.is_some_and(|expected| expected != sha256_hex(&bytes)) {
        return Err(format!(
            "scanned source changed after verification: {relative}"
        ));
    }
    Ok(())
}

pub(super) fn checked_path(root: &Path, path: &Path) -> Result<PathBuf, String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| format!("source path escapes checkout: {}", path.display()))?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(part) = component else {
            return Err(format!("source path is not canonical: {}", path.display()));
        };
        current.push(part);
        let metadata = std::fs::symlink_metadata(&current)
            .map_err(|error| format!("source path is missing: {}: {error}", path.display()))?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "source path contains a symlink: {}",
                path.display()
            ));
        }
    }
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| format!("source path is missing: {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() {
        return Err(format!(
            "source path must not be a symlink: {}",
            path.display()
        ));
    }
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("source path is unreadable: {}: {error}", path.display()))?;
    if !canonical.starts_with(root) || !canonical.is_file() {
        return Err(format!(
            "source path escapes checkout or is not a file: {}",
            path.display()
        ));
    }
    Ok(canonical)
}

pub(super) fn verify_checkout(root: &Path) -> Result<(), String> {
    super::super::operational_paths::verify_katana_revision(root)?;
    let status = git_output(root, &["status", "--porcelain=v1", "--untracked-files=all"])?;
    if !status.is_empty() {
        return Err("KatanA checkout must be clean".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::checked_path;
    use crate::capability_manifest::capability_manifest_test_fixtures::FixtureBuilder;

    #[test]
    fn source_path_requires_regular_in_root_file() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = FixtureBuilder::root()?;
        let root = fixture.path().canonicalize()?;
        let source = root.join("source.rs");
        std::fs::write(&source, "fn main() {}")?;
        assert_eq!(checked_path(&root, &source)?, source);
        assert!(checked_path(&root, &root).is_err());
        assert!(checked_path(&root, &root.join("missing.rs")).is_err());
        assert!(checked_path(&root, &root.join("../outside.rs")).is_err());
        let other = FixtureBuilder::root()?;
        let outside = other.path().canonicalize()?.join("source.rs");
        std::fs::write(&outside, "fn main() {}")?;
        assert!(checked_path(&root, &outside).is_err());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_source_and_ancestor_are_rejected() -> Result<(), Box<dyn std::error::Error>> {
        use std::os::unix::fs::symlink;

        let fixture = FixtureBuilder::root()?;
        let root = fixture.path().canonicalize()?;
        let directory = root.join("source");
        std::fs::create_dir(&directory)?;
        let source = directory.join("lib.rs");
        std::fs::write(&source, "fn main() {}")?;
        let file_alias = root.join("alias.rs");
        let directory_alias = root.join("alias");
        symlink(&source, &file_alias)?;
        symlink(&directory, &directory_alias)?;
        assert!(checked_path(&root, &file_alias).is_err());
        assert!(checked_path(&root, &directory_alias.join("lib.rs")).is_err());
        Ok(())
    }
}
