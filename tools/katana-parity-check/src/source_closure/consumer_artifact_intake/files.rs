use super::super::fingerprint::sha256_hex;
use super::model::{EvidenceStage, KUC_SCHEMA, KUC_SCHEMA_VERSION, KucManifest};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

const SHA256_HEX_LENGTH: usize = 64;
const UNICODE_EVIDENCE_DOMAIN: &[u8] = b"kuc.consumer-artifact.unicode-evidence.v1";

pub(super) fn validate_stage(
    root: &Path,
    stage: &EvidenceStage,
    receipts: &mut BTreeSet<String>,
) -> Result<(), String> {
    let media = safe_file(root, &stage.media_path, "media")?;
    let media_bytes = fs::read(&media).map_err(|error| format!("media cannot be read: {error}"))?;
    if !plain_sha256(&stage.media_sha256) || plain_hash(&media_bytes) != stage.media_sha256 {
        return Err(format!("media SHA-256 mismatch: {}", stage.media_path));
    }
    let manifest_name = format!("{}.consumer-artifact.json", stage.stage_id);
    let manifest: KucManifest = read_json(root, &manifest_name, "KUC stage manifest")?;
    validate_kuc_manifest(stage, &manifest)?;
    if !receipts.insert(manifest.forwarding_receipt_fingerprint.clone()) {
        return Err("consumer artifact evidence has duplicate receipt fingerprint".into());
    }
    let expected_unicode = format!("{}.unicode-evidence.json", stage.stage_id);
    if manifest.unicode_evidence_file != expected_unicode {
        return Err(format!(
            "Unicode evidence path differs for stage: {}",
            stage.stage_id
        ));
    }
    let unicode = safe_file(root, &manifest.unicode_evidence_file, "Unicode evidence")?;
    let unicode_bytes =
        fs::read(&unicode).map_err(|error| format!("Unicode evidence cannot be read: {error}"))?;
    if !plain_sha256(&manifest.unicode_evidence_payload_sha256)
        || plain_hash(&unicode_bytes) != manifest.unicode_evidence_payload_sha256
        || bound_unicode_hash(
            &unicode_bytes,
            stage,
            &manifest.forwarding_receipt_fingerprint,
        ) != manifest.unicode_evidence_hash
    {
        return Err(format!(
            "Unicode evidence binding mismatch: {}",
            stage.stage_id
        ));
    }
    Ok(())
}

fn validate_kuc_manifest(stage: &EvidenceStage, manifest: &KucManifest) -> Result<(), String> {
    if manifest.schema != KUC_SCHEMA || manifest.schema_version != KUC_SCHEMA_VERSION {
        return Err(format!(
            "invalid KUC stage manifest schema: {}",
            stage.stage_id
        ));
    }
    if manifest.stage_id != stage.stage_id
        || manifest.leaf_id != stage.leaf_id
        || manifest.root_revision != stage.root_revision
        || manifest.png_sha256 != stage.media_sha256
        || manifest.pixel_hash != stage.pixel_hash
        || manifest.root_record_hash != stage.root_record_hash
        || manifest.accesskit_snapshot_hash != stage.accesskit_snapshot_hash
        || manifest.unicode_evidence_hash != stage.unicode_evidence_hash
    {
        return Err(format!(
            "KUC stage manifest differs for stage: {}",
            stage.stage_id
        ));
    }
    if manifest.root_revision == 0
        || !plain_sha256(&manifest.png_sha256)
        || !plain_sha256(&manifest.pixel_hash)
        || !plain_sha256(&manifest.root_record_hash)
        || !plain_sha256(&manifest.accesskit_snapshot_hash)
        || !plain_sha256(&manifest.unicode_evidence_hash)
        || !plain_sha256(&manifest.unicode_evidence_payload_sha256)
        || !plain_sha256(&manifest.forwarding_receipt_fingerprint)
    {
        return Err(format!(
            "KUC stage manifest has invalid hashes: {}",
            stage.stage_id
        ));
    }
    Ok(())
}

pub(super) fn read_json<T: serde::de::DeserializeOwned>(
    root: &Path,
    relative: &str,
    kind: &str,
) -> Result<T, String> {
    let path = safe_file(root, relative, kind)?;
    let bytes = fs::read(path).map_err(|error| format!("{kind} cannot be read: {error}"))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("{kind} is invalid: {error}"))
}

pub(super) fn safe_root(path: &Path) -> Result<PathBuf, String> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| format!("artifact root is missing: {error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("artifact root must be a regular directory".into());
    }
    path.canonicalize()
        .map_err(|error| format!("artifact root cannot be canonicalized: {error}"))
}

fn safe_file(root: &Path, relative: &str, kind: &str) -> Result<PathBuf, String> {
    if relative.trim().is_empty() || relative.contains('\\') || Path::new(relative).is_absolute() {
        return Err(format!(
            "{kind} path is not a safe relative path: {relative:?}"
        ));
    }
    let mut current = root.to_path_buf();
    for component in Path::new(relative).components() {
        let Component::Normal(part) = component else {
            return Err(format!(
                "{kind} path is not a safe relative path: {relative:?}"
            ));
        };
        current.push(part);
        let metadata = fs::symlink_metadata(&current)
            .map_err(|error| format!("{kind} path is missing: {error}"))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("{kind} path contains a symlink: {relative}"));
        }
    }
    let metadata = fs::symlink_metadata(&current)
        .map_err(|error| format!("{kind} path is missing: {error}"))?;
    if !metadata.is_file() {
        return Err(format!("{kind} path is not a regular file: {relative}"));
    }
    let canonical = current
        .canonicalize()
        .map_err(|error| format!("{kind} path cannot be canonicalized: {error}"))?;
    if !canonical.starts_with(root) {
        return Err(format!("{kind} path escapes the artifact root: {relative}"));
    }
    Ok(canonical)
}

pub(super) fn prefixed_sha256(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(plain_sha256)
}

fn plain_sha256(value: &str) -> bool {
    value.len() == SHA256_HEX_LENGTH
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn plain_hash(bytes: &[u8]) -> String {
    sha256_hex(bytes).trim_start_matches("sha256:").to_owned()
}

fn bound_unicode_hash(bytes: &[u8], stage: &EvidenceStage, receipt: &str) -> String {
    let mut hasher = Sha256::new();
    for field in [
        UNICODE_EVIDENCE_DOMAIN,
        bytes,
        stage.stage_id.as_bytes(),
        stage.leaf_id.as_bytes(),
        &stage.root_revision.to_be_bytes(),
        stage.root_record_hash.as_bytes(),
        stage.accesskit_snapshot_hash.as_bytes(),
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
