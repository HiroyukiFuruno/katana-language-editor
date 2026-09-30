use std::fs;
use std::path::Path;

use super::fingerprint::sha256_hex;
use super::operational_input::FIXED_KATANA_REVISION;
use super::operational_loader::{SourceClosureInputLoaderVerifier, VerifiedSourceClosureInput};
use super::operational_output::{read_canonical_json, write_canonical_json};
use super::operational_publication_fingerprint::evidence_tree_fingerprint_for_staging;
use super::operational_staging::require_staging_run;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ValidationReceipt {
    schema_version: String,
    run_id: String,
    katana_revision: String,
    input_sha256: String,
    input_identity_fingerprint: String,
    evidence_tree_sha256: String,
}

pub(super) fn write_validation_receipt(
    staging: &Path,
    input: &Path,
    verified: &VerifiedSourceClosureInput,
) -> Result<(), String> {
    let run_id = verified
        .input()
        .root
        .evidence
        .katana_revision
        .capture_id
        .split("::")
        .nth(1)
        .ok_or_else(|| "input evidence has no run id".to_string())?;
    let bytes = fs::read(input).map_err(|error| format!("input is unreadable: {error}"))?;
    let receipt = ValidationReceipt {
        schema_version: "1".into(),
        run_id: run_id.into(),
        katana_revision: FIXED_KATANA_REVISION.into(),
        input_sha256: sha256_hex(&bytes),
        input_identity_fingerprint: verified.identity_fingerprint().into(),
        evidence_tree_sha256: evidence_tree_fingerprint_for_staging(staging)?,
    };
    write_canonical_json(
        &staging.join("validation/input-validation-receipt.json"),
        &receipt,
    )
}

pub(super) fn validate_validation_receipt(staging: &Path, input: &Path) -> Result<(), String> {
    let receipt: ValidationReceipt = read_canonical_json(
        &staging.join("validation/input-validation-receipt.json"),
        "validation receipt",
    )?;
    let (expected_staging, expected_run) = require_staging_run(input)?;
    let actual_staging = staging
        .canonicalize()
        .map_err(|error| format!("staging is unreadable: {error}"))?;
    if expected_staging != actual_staging
        || receipt.run_id != expected_run
        || receipt.katana_revision != FIXED_KATANA_REVISION
    {
        return Err("validation receipt is not bound to this staging run".into());
    }
    let bytes = fs::read(input).map_err(|error| format!("input is unreadable: {error}"))?;
    if receipt.input_sha256 != sha256_hex(&bytes)
        || receipt.evidence_tree_sha256 != evidence_tree_fingerprint_for_staging(staging)?
    {
        return Err("validation receipt does not match current staging".into());
    }
    let verified = SourceClosureInputLoaderVerifier::load(input)?;
    if receipt.input_identity_fingerprint != verified.identity_fingerprint() {
        return Err("validation receipt identity mismatch".into());
    }
    Ok(())
}
