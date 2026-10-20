use crate::source_closure::artifact_model::{LeafManifestArtifact, RequirementOrigin};
use crate::source_closure::branch_catalog::materialize_branch_catalog;
use crate::source_closure::canonical_leaf_binding::CanonicalLeafBindingEvidence;
use crate::source_closure::consumer_artifact_intake::{
    ConsumerArtifactIntake, ValidatedConsumerArtifact,
};
use crate::source_closure::fingerprint::sha256_hex;
use crate::source_closure::requirement_source_inventory::RequirementSourceInventoryReport;
use crate::source_closure::source_action_binding::SourceActionBindingReport;
use crate::source_closure::source_action_binding::tests::fixture;
use crate::source_closure::structured_leaf_candidates::StructuredLeafCandidatesReport;
use serde::de::DeserializeOwned;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
pub(super) const HASH_LEN: usize = 64;
static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);
pub(super) fn artifact(
    profile_id: &str,
    stage_id: &str,
) -> Result<(ValidatedConsumerArtifact, PathBuf), String> {
    let receipt_byte = match profile_id {
        "macos-latest" => 'e',
        "windows-latest" => 'f',
        _ => 'g',
    };
    artifact_with_receipt(profile_id, stage_id, receipt_byte)
}
pub(super) fn artifact_with_receipt(
    profile_id: &str,
    stage_id: &str,
    receipt_byte: char,
) -> Result<(ValidatedConsumerArtifact, PathBuf), String> {
    let root = temp_dir(&format!("records-artifact-{}-{}", profile_id, stage_id))?;
    let media = b"png-a";
    let unicode = b"unicode-a";
    let root_record = "c".repeat(HASH_LEN);
    let accesskit = "d".repeat(HASH_LEN);
    let receipt = receipt_byte.to_string().repeat(HASH_LEN);
    let unicode_hash = bound_hash(
        unicode,
        stage_id,
        "leaf-a",
        &root_record,
        &accesskit,
        &receipt,
    );
    write(&root, &format!("{}.png", stage_id), media)?;
    write(
        &root,
        &format!("{}.unicode-evidence.json", stage_id),
        unicode,
    )?;
    write_json(
        &root,
        &format!("{}.consumer-artifact.json", stage_id),
        json!({
            "schema":"kuc.consumer-full-editor-artifact.v1", "schema_version":1,
            "stage_id":stage_id, "leaf_id":"leaf-a", "root_revision":1,
            "png_sha256":plain_hash(media), "pixel_hash":"a".repeat(HASH_LEN),
            "root_record_hash":root_record, "accesskit_snapshot_hash":accesskit,
            "unicode_evidence_hash":unicode_hash,
            "unicode_evidence_payload_sha256":plain_hash(unicode),
            "unicode_evidence_file":format!("{}.unicode-evidence.json", stage_id),
            "forwarding_receipt_fingerprint":receipt
        }),
    )?;
    write_json(
        &root,
        "kle-consumer-artifact-evidence.json",
        json!({
            "schema":"kle-consumer-artifact-evidence.v1", "run_id":"run-1",
            "source_closure_sha256":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "profile_fingerprint":"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "profile_id":profile_id, "stages":[{
                "leaf_id":"leaf-a", "interaction":"text_input", "stage_id":stage_id,
                "media_path":format!("{}.png", stage_id), "media_sha256":plain_hash(media),
                "pixel_hash":"a".repeat(HASH_LEN), "root_record_hash":root_record,
                "accesskit_snapshot_hash":accesskit, "unicode_evidence_hash":unicode_hash,
                "root_revision":1
            }]
        }),
    )?;
    let plan = serde_json::to_vec(&json!({
        "schema":"kle.consumer-artifact-plan.v1", "run_id":"run-1",
        "source_closure_sha256":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "profile_fingerprint":"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "profile_id":profile_id, "stages":[{"leaf_id":"leaf-a","interaction":"text_input"}]
    }))
    .map_err(|error| error.to_string())?;
    Ok((
        ConsumerArtifactIntake::validate_profile_artifact(&plan, &root)?,
        root,
    ))
}
pub(super) fn manifest() -> Result<(LeafManifestArtifact, PathBuf), String> {
    let facts = fixture::actual_fixture().map_err(|error| error.to_string())?;
    let branches =
        materialize_branch_catalog(facts.root.clone(), &facts.state, &[], facts._fixture.path())?;
    let t3 = SourceActionBindingReport::build(&facts.state, &facts.source, &facts.requirements)?;
    let structured =
        StructuredLeafCandidatesReport::build(&facts.source, &branches, &facts.requirements, &t3)?;
    let binding = CanonicalLeafBindingEvidence {
        source_analysis_revision:
            crate::source_closure::canonical_leaf_binding::SOURCE_ANALYSIS_REVISION.into(),
        inventory: RequirementSourceInventoryReport {
            requirements_sha256: "fixture".into(),
            rows: vec![],
            entries: vec![],
        },
        t2: facts.requirements,
        t3,
        structured,
    };
    let mut leafs = binding.project_leafs(&branches)?;
    let leaf = leafs.first_mut().ok_or("fixture leaf is missing")?;
    leaf.leaf_id = "leaf-a".into();
    leaf.required_profile_ids = vec!["macos-latest".into(), "windows-latest".into()];
    leaf.requirement_origin = RequirementOrigin {
        kind: "test".into(),
        span: "test:1".into(),
    };
    leafs.truncate(1);
    let root = temp_dir("records-manifest")?;
    Ok((
        LeafManifestArtifact {
            root: facts.root,
            canonical_binding: binding,
            leafs,
        },
        root,
    ))
}
pub(super) fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, String> {
    serde_json::from_slice(&fs::read(path).map_err(|error| error.to_string())?)
        .map_err(|error| error.to_string())
}

pub(super) fn temp_dir(label: &str) -> Result<PathBuf, String> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("kle-{}-{}-{}", label, nonce, id));
    fs::create_dir(&path).map_err(|error| error.to_string())?;
    Ok(path)
}

pub(super) fn cleanup(path: PathBuf) -> Result<(), String> {
    fs::remove_dir_all(path).map_err(|error| error.to_string())
}

fn write(root: &Path, name: &str, bytes: &[u8]) -> Result<(), String> {
    fs::write(root.join(name), bytes).map_err(|error| error.to_string())
}

fn write_json(root: &Path, name: &str, value: serde_json::Value) -> Result<(), String> {
    write(
        root,
        name,
        &serde_json::to_vec(&value).map_err(|error| error.to_string())?,
    )
}

fn plain_hash(bytes: &[u8]) -> String {
    sha256_hex(bytes).trim_start_matches("sha256:").to_owned()
}

fn bound_hash(
    bytes: &[u8],
    stage: &str,
    leaf: &str,
    root: &str,
    accesskit: &str,
    receipt: &str,
) -> String {
    let mut hasher = Sha256::new();
    for field in [
        b"kuc.consumer-artifact.unicode-evidence.v1".as_slice(),
        bytes,
        stage.as_bytes(),
        leaf.as_bytes(),
        &1_u64.to_be_bytes(),
        root.as_bytes(),
        accesskit.as_bytes(),
        receipt.as_bytes(),
    ] {
        hasher.update((field.len() as u64).to_be_bytes());
        hasher.update(field);
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
