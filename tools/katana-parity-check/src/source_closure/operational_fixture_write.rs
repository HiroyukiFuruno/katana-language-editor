use std::fs;
use std::path::Path;

use super::fingerprint::sha256_hex;
use super::operational_input::EvidenceRef;

pub(super) fn write_evidence(
    input_dir: &Path,
    relative: &str,
    bytes: &[u8],
) -> Result<EvidenceRef, String> {
    let path = input_dir.join(relative);
    let parent = path
        .parent()
        .ok_or_else(|| format!("evidence path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "evidence parent create failed for {}: {error}",
            parent.display()
        )
    })?;
    fs::write(&path, bytes).map_err(|error| {
        format!(
            "evidence bytes write failed for {}: {error}",
            path.display()
        )
    })?;
    Ok(EvidenceRef {
        capture_id: format!("run::fixture::{relative}"),
        path: relative.into(),
        sha256: sha256_hex(bytes),
        command_or_source: "fixture-capture".into(),
        runner_label: "read-only-local-source".into(),
        exit_status: 0,
    })
}
