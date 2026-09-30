use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use super::operational_cli::PROFILE_IDS;
use super::operational_input::{EvidenceRef, ProfileProbeInput};
use super::operational_output::{ChecksumsFile, read_canonical_json};

const CAPTURE_ID_PARTS: usize = 3;

pub(super) fn reject_symlink(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("staging path is missing or unreadable: {error}"))?;
    if metadata.file_type().is_symlink() {
        return Err(format!(
            "symlink is forbidden in source-closure staging: {}",
            path.display()
        ));
    }
    Ok(())
}

pub(super) fn validate_profile_staging(staging: &Path) -> Result<(), String> {
    let run_id = staging
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "staging directory has no run id".to_string())?;
    validate_profile_layout(staging, run_id)
}

pub(super) fn validate_profile_layout(staging: &Path, run_id: &str) -> Result<(), String> {
    reject_symlink(staging)?;
    let profiles = staging.join("profiles");
    require_dir(&profiles)?;
    let actual = names(&profiles)?;
    let expected = PROFILE_IDS
        .iter()
        .map(|id| (*id).to_string())
        .collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(format!("profile artifact set mismatch: {actual:?}"));
    }
    for id in PROFILE_IDS {
        validate_profile(&profiles.join(id), id, run_id)?;
    }
    Ok(())
}

fn validate_profile(dir: &Path, id: &str, run_id: &str) -> Result<(), String> {
    require_dir(dir)?;
    let expected = ["checksums.json", "probe.json", "raw"]
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    if names(dir)? != expected {
        return Err(format!(
            "profile {id} tree is partial or contains an unknown entry"
        ));
    }
    let probe: ProfileProbeInput = read_canonical_json(&dir.join("probe.json"), "profile")?;
    if probe.id != id || probe.runner_label != id {
        return Err(format!("profile {id} identity mismatch"));
    }
    let checksums: ChecksumsFile = read_canonical_json(&dir.join("checksums.json"), "checksums")?;
    if checksums.profile_id != id {
        return Err(format!("profile {id} checksum identity mismatch"));
    }
    let raw = dir.join("raw");
    require_dir(&raw)?;
    let files = files(&raw)?;
    if files.is_empty() {
        return Err(format!("profile {id} raw evidence is empty"));
    }
    let actual = files
        .into_iter()
        .map(|path| {
            let relative = path
                .strip_prefix(staging_root(dir))
                .map_err(|_| "profile path escaped staging".to_string())?
                .to_string_lossy()
                .replace(std::path::MAIN_SEPARATOR, "/");
            let bytes =
                fs::read(&path).map_err(|error| format!("profile evidence unreadable: {error}"))?;
            Ok((relative, super::fingerprint::sha256_hex(&bytes)))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    let expected = checksums
        .entries
        .into_iter()
        .map(|entry| (entry.path, entry.sha256))
        .collect::<BTreeMap<_, _>>();
    if actual != expected {
        return Err(format!("profile {id} checksums do not match raw evidence"));
    }
    validate_profile_evidence(&probe, id, run_id)
}

fn validate_profile_evidence(
    probe: &ProfileProbeInput,
    id: &str,
    run_id: &str,
) -> Result<(), String> {
    for evidence in evidence_refs(probe) {
        let parts = evidence
            .capture_id
            .splitn(CAPTURE_ID_PARTS, "::")
            .collect::<Vec<_>>();
        if parts.len() != CAPTURE_ID_PARTS
            || parts[0] != "run"
            || parts[1] != run_id
            || evidence.runner_label != id
        {
            return Err(format!(
                "profile {id} contains cross-run or foreign evidence"
            ));
        }
    }
    Ok(())
}

fn evidence_refs(probe: &ProfileProbeInput) -> Vec<&EvidenceRef> {
    let mut refs = vec![
        &probe.rustc_vv_raw,
        &probe.rustc_cfg_raw,
        &probe.cargo_resolution_raw,
        &probe.cargo_lock_raw,
        &probe.cfg_edge_probe,
    ];
    refs.extend(&probe.source_tree);
    refs
}

fn staging_root(path: &Path) -> &Path {
    path.parent().and_then(Path::parent).unwrap_or(path)
}

fn require_dir(path: &Path) -> Result<(), String> {
    reject_symlink(path)?;
    if !path.is_dir() {
        return Err(format!(
            "required staging directory is missing: {}",
            path.display()
        ));
    }
    Ok(())
}

fn names(path: &Path) -> Result<BTreeSet<String>, String> {
    fs::read_dir(path)
        .map_err(|error| format!("failed to enumerate staging: {error}"))?
        .map(|entry| {
            let entry = entry.map_err(|error| format!("failed to read staging entry: {error}"))?;
            reject_symlink(&entry.path())?;
            entry
                .file_name()
                .to_str()
                .map(str::to_string)
                .ok_or_else(|| "staging path is not UTF-8".to_string())
        })
        .collect()
}

fn files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut result = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        reject_symlink(&dir)?;
        for entry in fs::read_dir(&dir)
            .map_err(|error| format!("failed to enumerate raw evidence: {error}"))?
        {
            let entry = entry.map_err(|error| format!("failed to read raw evidence: {error}"))?;
            let path = entry.path();
            reject_symlink(&path)?;
            collect_file_entry(path, &mut pending, &mut result)?;
        }
    }
    result.sort();
    Ok(result)
}

fn collect_file_entry(
    path: PathBuf,
    pending: &mut Vec<PathBuf>,
    result: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if path.is_dir() {
        pending.push(path);
        return Ok(());
    }
    if path.is_file() {
        result.push(path);
        return Ok(());
    }
    Err("special file in raw evidence".into())
}
