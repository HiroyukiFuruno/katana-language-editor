use std::{fs, path::Path};

use crate::source_closure::fingerprint::sha256_hex;
use crate::source_closure::operational_input::{EvidenceRef, FIXED_KATANA_REVISION};

pub(super) fn capture(input_dir: &Path) -> Result<EvidenceRef, String> {
    let relative = "raw/provenance/source-universe.md";
    let path = input_dir.join(relative);
    let parent = path
        .parent()
        .ok_or_else(|| format!("fixture source-universe has no parent: {}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("fixture source-universe parent create failed: {error}"))?;
    let bytes =
        format!("# KatanA Editor Source Universe\n\n{FIXED_KATANA_REVISION}\n").into_bytes();
    fs::write(&path, &bytes)
        .map_err(|error| format!("fixture source-universe write failed: {error}"))?;
    Ok(EvidenceRef {
        capture_id: format!("run::fixture::{relative}"),
        path: relative.into(),
        sha256: sha256_hex(&bytes),
        command_or_source: "fixture-capture".into(),
        runner_label: "read-only-local-source".into(),
        exit_status: 0,
    })
}
