use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use super::operational_input::FIXED_KATANA_REVISION;
use crate::system::ProcessService;

const KATANA_REFERENCE_REPOSITORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../katana");

pub(super) fn fixed_reference_root() -> Result<&'static Path, String> {
    static CACHE: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    match CACHE.get_or_init(build_fixed_reference_root) {
        Ok(path) => Ok(path.as_path()),
        Err(error) => Err(error.clone()),
    }
}

fn build_fixed_reference_root() -> Result<PathBuf, String> {
    let destination = std::env::temp_dir().join(format!(
        "kpc-fixed-katana-{}-{}-{}",
        std::process::id(),
        FIXED_KATANA_REVISION,
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| format!("fixed reference clock failed: {error}"))?
            .as_nanos()
    ));
    create_fixed_reference_checkout(&destination)?;
    let canonical = destination
        .canonicalize()
        .map_err(|error| format!("fixed reference root canonicalize failed: {error}"))?;
    seal_read_only(&canonical)?;
    Ok(canonical)
}

fn create_fixed_reference_checkout(destination: &Path) -> Result<(), String> {
    let clone = ProcessService::create_command("git")
        .args(["clone", "--quiet", "--no-checkout", "--no-local"])
        .arg(KATANA_REFERENCE_REPOSITORY)
        .arg(destination)
        .output()
        .map_err(|error| format!("fixed reference clone failed to start: {error}"))?;
    if !clone.status.success() {
        return Err(format!(
            "fixed reference clone failed: {}",
            String::from_utf8_lossy(&clone.stderr).trim()
        ));
    }
    let checkout = ProcessService::create_command("git")
        .args(["-C"])
        .arg(destination)
        .args(["checkout", "--quiet", "--detach", FIXED_KATANA_REVISION])
        .output()
        .map_err(|error| format!("fixed reference checkout failed to start: {error}"))?;
    if !checkout.status.success() {
        return Err(format!(
            "fixed reference checkout failed: {}",
            String::from_utf8_lossy(&checkout.stderr).trim()
        ));
    }
    Ok(())
}

fn seal_read_only(root: &Path) -> Result<(), String> {
    let mut paths = vec![root.to_path_buf()];
    while let Some(path) = paths.pop() {
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            format!(
                "fixed reference metadata failed for {}: {error}",
                path.display()
            )
        })?;
        let mut permissions = metadata.permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&path, permissions).map_err(|error| {
            format!(
                "fixed reference permissions failed for {}: {error}",
                path.display()
            )
        })?;
        if metadata.is_dir() {
            for entry in fs::read_dir(&path).map_err(|error| {
                format!(
                    "fixed reference directory read failed for {}: {error}",
                    path.display()
                )
            })? {
                let entry = entry.map_err(|error| {
                    format!(
                        "fixed reference directory entry failed for {}: {error}",
                        path.display()
                    )
                })?;
                paths.push(entry.path());
            }
        }
    }
    Ok(())
}
