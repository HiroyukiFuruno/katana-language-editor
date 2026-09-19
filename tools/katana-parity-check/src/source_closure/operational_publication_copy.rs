use std::fs;
use std::path::Path;

pub(super) fn copy_tree(source: &Path, destination: &Path) -> Result<(), String> {
    if !source.is_dir() {
        return Err(format!(
            "publication source is missing: {}",
            source.display()
        ));
    }
    fs::create_dir_all(destination)
        .map_err(|error| format!("failed to create publication directory: {error}"))?;
    for entry in fs::read_dir(source)
        .map_err(|error| format!("failed to read publication source: {error}"))?
    {
        let entry = entry.map_err(|error| format!("failed to read publication source: {error}"))?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let metadata = fs::symlink_metadata(&source_path)
            .map_err(|error| format!("publication source is unreadable: {error}"))?;
        copy_entry(&source_path, &destination_path, metadata, destination)?;
    }
    Ok(())
}

fn copy_entry(
    source: &Path,
    destination: &Path,
    metadata: fs::Metadata,
    parent: &Path,
) -> Result<(), String> {
    if metadata.file_type().is_symlink() {
        return Err(format!(
            "publication source contains a symlink: {}",
            source.display()
        ));
    }
    if metadata.is_dir() {
        copy_tree(source, destination)
    } else if metadata.is_file() {
        fs::create_dir_all(destination.parent().unwrap_or(parent))
            .map_err(|error| format!("failed to create publication directory: {error}"))?;
        fs::copy(source, destination).map_err(|error| {
            format!(
                "failed to copy publication evidence {} -> {}: {error}",
                source.display(),
                destination.display()
            )
        })?;
        Ok(())
    } else {
        Err(format!(
            "publication source contains a special file: {}",
            source.display()
        ))
    }
}
