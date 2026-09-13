use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::fingerprint::sha256_hex;
use super::operational_input::SourceClosureInput;
use super::operational_loader::VerifiedSourceClosureInput;
use super::operational_output::write_canonical_json;
use super::operational_publication_copy::copy_tree;
use super::operational_publication_fingerprint::{
    evidence_tree_fingerprint_for, evidence_tree_fingerprint_for_staging,
};
use super::operational_publication_materialized::{
    copy_verified_materialized_artifacts, verify_published_materialized_artifacts,
};

pub(super) fn publish(
    staging: &Path,
    input_path: &Path,
    verified: &VerifiedSourceClosureInput,
    artifact_dir: &Path,
) -> Result<PathBuf, String> {
    super::operational_staging::validate_validation_receipt(staging, input_path)?;
    let revision = &verified.input().root.katana_revision;
    let parent = artifact_dir.join("artifacts/v0-1-0/source-closure-input");
    fs::create_dir_all(&parent)
        .map_err(|error| format!("failed to create publication parent: {error}"))?;
    let destination = parent.join(revision);
    let temp = parent.join(format!(".{revision}.{}-tmp", unique_suffix()));
    if temp.exists() || destination.exists() && !destination.is_dir() {
        return Err(
            "publication destination is occupied by a non-directory or stale temporary".into(),
        );
    }
    if destination.is_dir() {
        verify_existing(&destination, staging, input_path)?;
        return Ok(destination);
    }

    let staging_evidence_hash = evidence_tree_fingerprint_for_staging(staging)?;
    fs::create_dir(&temp)
        .map_err(|error| format!("failed to create publication temporary: {error}"))?;
    let result = (|| {
        copy_tree(&staging.join("profiles"), &temp.join("evidence/profiles"))?;
        copy_tree(
            &staging.join("provenance"),
            &temp.join("evidence/provenance"),
        )?;
        let input = canonical_publication_input(input_path)?;
        let input_bytes = input.canonical_json_bytes()?;
        fs::write(temp.join("source-closure-input.json"), &input_bytes)
            .map_err(|error| format!("failed to write publication input: {error}"))?;
        let evidence_hash = evidence_tree_fingerprint_for(&temp.join("evidence"))?;
        if evidence_hash != staging_evidence_hash {
            return Err("publication evidence does not match staging".into());
        }
        let materialized_artifacts_sha256 = copy_verified_materialized_artifacts(staging, &temp)?;
        let receipt = serde_json::json!({
            "schema_version": "1",
            "run_id": capture_run_id(verified)?,
            "katana_revision": revision,
            "input_sha256": sha256_hex(&input_bytes),
            "input_identity_fingerprint": verified.identity_fingerprint(),
            "evidence_tree_sha256": evidence_hash,
            "materialized_artifacts_sha256": materialized_artifacts_sha256,
        });
        write_canonical_json(
            &temp.join("validation/input-validation-receipt.json"),
            &receipt,
        )?;
        super::operational_loader::SourceClosureInputLoaderVerifier::load(
            &temp.join("source-closure-input.json"),
        )?;
        fs::rename(&temp, &destination)
            .map_err(|error| format!("failed to atomically publish source-closure input: {error}"))
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&temp);
    }
    result.map(|_| destination)
}

fn canonical_publication_input(path: &Path) -> Result<SourceClosureInput, String> {
    let mut value: serde_json::Value = serde_json::from_slice(
        &fs::read(path).map_err(|error| format!("failed to read assembled input: {error}"))?,
    )
    .map_err(|error| format!("assembled input is invalid: {error}"))?;
    rewrite_evidence_paths(&mut value);
    let mut input: SourceClosureInput = serde_json::from_value(value)
        .map_err(|error| format!("publication input is invalid: {error}"))?;
    input.root.katana_external_ui_fingerprint =
        super::operational_input::EvidenceRef::set_fingerprint(
            &input.root.evidence.katana_external_ui,
        );
    input.root.kle_tree_fingerprint =
        super::operational_input::EvidenceRef::set_fingerprint(&input.root.evidence.kle_tree);
    input.root.kuc_tree_fingerprint =
        super::operational_input::EvidenceRef::set_fingerprint(&input.root.evidence.kuc_tree);
    input.root.generator_fingerprint = super::operational_input::EvidenceRef::set_fingerprint(&[
        input.root.evidence.generator_binary.clone(),
        input.root.evidence.generator_schema.clone(),
    ]);
    for profile in &mut input.profile_probes {
        profile.source_tree_fingerprint =
            super::operational_input::EvidenceRef::profile_tree_fingerprint(
                &profile.id,
                &profile.source_tree,
            );
        profile.fingerprint = profile.identity_fingerprint();
    }
    input.root.release_profile_matrix_fingerprint = input.matrix_fingerprint();
    Ok(input)
}

fn rewrite_evidence_paths(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, value) in map.iter_mut() {
                if key == "path"
                    && let serde_json::Value::String(path) = value
                    && (path.starts_with("profiles/") || path.starts_with("provenance/"))
                {
                    *path = format!("evidence/{path}");
                }
                rewrite_evidence_paths(value);
            }
        }
        serde_json::Value::Array(values) => values.iter_mut().for_each(rewrite_evidence_paths),
        _ => {}
    }
}

fn verify_existing(destination: &Path, staging: &Path, input_path: &Path) -> Result<(), String> {
    let existing = super::operational_loader::SourceClosureInputLoaderVerifier::load(
        &destination.join("source-closure-input.json"),
    )?;
    let candidate = canonical_publication_input(input_path)?;
    if existing.input().canonical_json_bytes()? != candidate.canonical_json_bytes()? {
        return Err("canonical source-closure revision exists with mismatched content".into());
    }
    if evidence_tree_fingerprint_for(&destination.join("evidence"))?
        != evidence_tree_fingerprint_for_staging(staging)?
    {
        return Err("canonical source-closure evidence does not match staging".into());
    }
    verify_published_materialized_artifacts(staging, destination)?;
    Ok(())
}

fn capture_run_id(verified: &VerifiedSourceClosureInput) -> Result<String, String> {
    verified
        .input()
        .root
        .evidence
        .katana_revision
        .capture_id
        .split("::")
        .nth(1)
        .map(str::to_string)
        .ok_or_else(|| "input evidence has no run id".into())
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos())
}
