use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use super::fingerprint::sha256_hex;
use super::operational_cli::PROFILE_IDS;
use super::operational_input::EvidenceRef;
use super::operational_process::{git_files, read_repo_file};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ChecksumsFile {
    pub(super) schema_version: String,
    pub(super) profile_id: String,
    pub(super) entries: Vec<ChecksumEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ChecksumEntry {
    pub(super) path: String,
    pub(super) sha256: String,
}

pub(super) fn capture_tree(
    output_root: &Path,
    repo_root: &Path,
    name: &str,
    run_id: &str,
    evidence: &mut Vec<EvidenceRef>,
) -> Result<Vec<EvidenceRef>, String> {
    let paths = git_files(
        repo_root,
        &[
            "*.rs",
            "Cargo.toml",
            "Cargo.lock",
            "Justfile",
            "clippy.toml",
        ],
    )?;
    if paths.is_empty() {
        return Err(format!("{name} source/config/lock capture is empty"));
    }
    paths
        .iter()
        .map(|path| {
            let bytes = read_repo_file(repo_root, path)?;
            capture_bytes(
                output_root,
                &format!("provenance/{name}/{path}"),
                &bytes,
                run_id,
                "read-only-local-source",
                &format!("{name}/{path}"),
                evidence,
            )
        })
        .collect()
}

pub(super) fn capture_bytes(
    output_root: &Path,
    relative_path: &str,
    bytes: &[u8],
    run_id: &str,
    runner_label: &str,
    command_or_source: &str,
    evidence: &mut Vec<EvidenceRef>,
) -> Result<EvidenceRef, String> {
    let path = output_root.join(relative_path);
    write_new(&path, bytes)?;
    let scope = if PROFILE_IDS.contains(&runner_label) {
        format!("profile::{runner_label}")
    } else {
        "provenance".into()
    };
    let capture_id = format!("run::{run_id}::{scope}::{relative_path}");
    let entry = EvidenceRef {
        capture_id,
        path: relative_path.into(),
        sha256: sha256_hex(bytes),
        command_or_source: command_or_source.into(),
        runner_label: runner_label.into(),
        exit_status: 0,
    };
    evidence.push(entry.clone());
    Ok(entry)
}

pub(super) fn write_checksums(
    path: &Path,
    profile_id: &str,
    evidence: &[EvidenceRef],
) -> Result<(), String> {
    let mut entries = evidence
        .iter()
        .map(|entry| ChecksumEntry {
            path: entry.path.clone(),
            sha256: entry.sha256.clone(),
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    write_canonical_json(
        path,
        &ChecksumsFile {
            schema_version: "1".into(),
            profile_id: profile_id.into(),
            entries,
        },
    )
}

pub(super) fn write_canonical_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("failed to serialize JSON {}: {error}", path.display()))?;
    bytes.push(b'\n');
    write_new(path, &bytes)
}

pub(super) fn read_canonical_json<T: for<'de> Deserialize<'de> + Serialize>(
    path: &Path,
    label: &str,
) -> Result<T, String> {
    let bytes = fs::read(path).map_err(|error| format!("{label} is missing: {error}"))?;
    let value: T =
        serde_json::from_slice(&bytes).map_err(|error| format!("invalid {label} JSON: {error}"))?;
    let mut canonical = serde_json::to_vec_pretty(&value)
        .map_err(|error| format!("failed to canonicalize {label}: {error}"))?;
    canonical.push(b'\n');
    if bytes != canonical {
        return Err(format!("{label} is not canonical JSON"));
    }
    Ok(value)
}

pub(super) fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if path.exists() {
        return Err(format!(
            "refusing to overwrite immutable output: {}",
            path.display()
        ));
    }
    fs::create_dir_all(path.parent().unwrap_or(Path::new(".")))
        .map_err(|error| format!("failed to create output directory: {error}"))?;
    fs::write(path, bytes).map_err(|error| format!("failed to write {}: {error}", path.display()))
}
