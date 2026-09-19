use std::path::{Component, Path, PathBuf};

use super::fingerprint::sha256_hex;
use super::operational_input::EvidenceRef;
use super::operational_validation::InputVerifier;

const HASH_HEX_LENGTH: usize = 64;
const CAPTURE_ID_PARTS: usize = 3;

pub(super) use super::operational_evidence_validation::validate_timestamp;

impl<'a> InputVerifier<'a> {
    pub(super) fn read_evidence(
        &mut self,
        evidence: &EvidenceRef,
        expected_runner: &str,
    ) -> Result<Vec<u8>, String> {
        validate_evidence_shape(evidence, expected_runner)?;
        self.verify_capture_run(&evidence.capture_id)?;
        if let Some(owner) = self
            .seen_capture_ids
            .insert(evidence.capture_id.clone(), expected_runner.into())
        {
            return Err(format!(
                "evidence capture id reused between {owner} and {expected_runner}"
            ));
        }
        if let Some(owner) = self
            .seen_paths
            .insert(evidence.path.clone(), expected_runner.into())
        {
            return Err(format!(
                "evidence path reused between {owner} and {expected_runner}"
            ));
        }
        let path = safe_evidence_path(self.evidence_dir, self.input_json, &evidence.path)?;
        let bytes = std::fs::read(&path)
            .map_err(|error| format!("missing evidence {}: {error}", evidence.path))?;
        if sha256_hex(&bytes) != evidence.sha256 {
            return Err(format!("evidence SHA-256 mismatch: {}", evidence.path));
        }
        Ok(bytes)
    }

    fn verify_capture_run(&mut self, capture_id: &str) -> Result<(), String> {
        let mut parts = capture_id.splitn(CAPTURE_ID_PARTS, "::");
        if parts.next() != Some("run") {
            return Err(format!(
                "evidence capture id must start with run::<id>::: {capture_id}"
            ));
        }
        let Some(run_id) = parts.next() else {
            return Err(format!("evidence capture id has no run id: {capture_id}"));
        };
        if run_id.trim().is_empty() || parts.next().is_none() {
            return Err(format!(
                "evidence capture id has invalid run scope: {capture_id}"
            ));
        }
        if let Some(expected) = &self.capture_run_id {
            if expected != run_id {
                return Err(format!(
                    "evidence mixes capture runs: expected {expected}, got {run_id}"
                ));
            }
        } else {
            self.capture_run_id = Some(run_id.to_string());
        }
        Ok(())
    }
}

fn validate_evidence_shape(evidence: &EvidenceRef, expected_runner: &str) -> Result<(), String> {
    validate_relative_path(&evidence.path)?;
    if evidence.capture_id.trim().is_empty()
        || is_placeholder(&evidence.capture_id)
        || is_placeholder(&evidence.command_or_source)
        || is_placeholder(&evidence.runner_label)
        || evidence.command_or_source.trim().is_empty()
        || evidence.runner_label != expected_runner
    {
        return Err(format!("invalid evidence metadata for {}", evidence.path));
    }
    if evidence.exit_status != 0 {
        return Err(format!("evidence command failed: {}", evidence.path));
    }
    validate_hash(&evidence.sha256, &format!("evidence {}", evidence.path))
}

fn safe_evidence_path(base: &Path, input_json: &Path, relative: &str) -> Result<PathBuf, String> {
    let base = base
        .canonicalize()
        .map_err(|error| format!("evidence directory is unreadable: {error}"))?;
    let candidate = base.join(relative);
    let mut current = base.clone();
    for component in Path::new(relative).components() {
        if let Component::Normal(part) = component {
            current.push(part);
            let metadata = std::fs::symlink_metadata(&current)
                .map_err(|error| format!("evidence path is missing: {relative}: {error}"))?;
            if metadata.file_type().is_symlink() {
                return Err(format!("evidence path contains a symlink: {relative}"));
            }
        }
    }
    let canonical = base
        .join(candidate.strip_prefix(&base).unwrap_or(Path::new(relative)))
        .canonicalize()
        .map_err(|error| format!("evidence path is missing: {relative}: {error}"))?;
    if !canonical.starts_with(&base) {
        return Err(format!("evidence path escapes input directory: {relative}"));
    }
    if std::fs::symlink_metadata(base.join(relative))
        .map_err(|error| format!("evidence path is unreadable: {relative}: {error}"))?
        .file_type()
        .is_symlink()
    {
        return Err(format!("evidence path is a symlink: {relative}"));
    }
    if canonical == input_json {
        return Err("input JSON cannot be used as raw evidence".into());
    }
    Ok(canonical)
}

pub(super) fn validate_relative_path(value: &str) -> Result<(), String> {
    if value.trim().is_empty() || value.contains('\\') {
        return Err(format!("path is not canonical relative: {value:?}"));
    }
    let path = Path::new(value);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir
                    | Component::CurDir
                    | Component::RootDir
                    | Component::Prefix(_)
            )
        })
    {
        return Err(format!("path is not canonical relative: {value:?}"));
    }
    Ok(())
}

pub(super) fn validate_rust_source_path(value: &str) -> Result<(), String> {
    validate_relative_path(value)?;
    if Path::new(value)
        .extension()
        .and_then(|extension| extension.to_str())
        != Some("rs")
    {
        return Err(format!("KatanA tree source path must be Rust: {value:?}"));
    }
    Ok(())
}

pub(super) fn validate_hash(value: &str, field: &str) -> Result<(), String> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err(format!("{field} must use sha256:<64 lowercase hex>"));
    };
    if hex.len() != HASH_HEX_LENGTH
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(format!("{field} must use sha256:<64 lowercase hex>"));
    }
    Ok(())
}

fn is_placeholder(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "placeholder" | "todo" | "tbd" | "stub" | "dummy" | "default" | "unknown" | "n/a"
    )
}
