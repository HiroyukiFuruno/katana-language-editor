use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn rust_paths(root: &Path) -> Result<BTreeSet<String>, String> {
    let metadata = fs::symlink_metadata(root)
        .map_err(|error| format!("external source root is unavailable: {error}"))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(format!(
            "external source root is not a directory: {}",
            root.display()
        ));
    }
    let mut paths = BTreeSet::new();
    visit(root, PathBuf::new(), &mut paths)?;
    Ok(paths)
}

fn visit(root: &Path, relative: PathBuf, paths: &mut BTreeSet<String>) -> Result<(), String> {
    for entry in fs::read_dir(root.join(&relative))
        .map_err(|error| format!("external source root cannot be read: {error}"))?
    {
        let entry =
            entry.map_err(|error| format!("external source root cannot be read: {error}"))?;
        let name = entry.file_name();
        let name = name
            .to_str()
            .ok_or_else(|| "external source root has a non-UTF-8 path".to_owned())?;
        if name.contains('\\') {
            return Err(format!(
                "external source root has an invalid path: {name:?}"
            ));
        }
        let child = relative.join(name);
        let metadata = fs::symlink_metadata(entry.path())
            .map_err(|error| format!("external source path is unavailable: {error}"))?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "external source root contains a symlink: {}",
                child.display()
            ));
        }
        if metadata.is_dir() {
            visit(root, child, paths)?;
        } else if metadata.is_file() && name.ends_with(".rs") {
            let path = child
                .to_str()
                .ok_or_else(|| "external source root has a non-UTF-8 path".to_owned())?
                .replace(std::path::MAIN_SEPARATOR, "/");
            paths.insert(path);
        }
    }
    Ok(())
}
