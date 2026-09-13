use super::digest::{decode_digest, is_digest};
use super::types::{
    SourceDerivedNativeTarget, SourceDerivedNativeTargetError, SourceDerivedNativeTargetRecord,
};
use crate::ax_target_locator::AxTargetLocator;
use crate::fixed_source_harness::FIXED_KATANA_REVISION;
use crate::system::ProcessService;
use std::path::Path;

const RECORD_SCHEMA_VERSION: &str = "1";
const GENERATED_BY: &str = "katana-source-closure-generator";

impl SourceDerivedNativeTarget {
    pub fn load(
        record_path: impl AsRef<Path>,
        fixed_source: impl AsRef<Path>,
        source_closure_profile: impl AsRef<Path>,
    ) -> Result<AxTargetLocator, SourceDerivedNativeTargetError> {
        let record_path = record_path.as_ref();
        let bytes = std::fs::read(record_path).map_err(|source| {
            SourceDerivedNativeTargetError::ReadRecord {
                path: record_path.to_path_buf(),
                source,
            }
        })?;
        let record: SourceDerivedNativeTargetRecord =
            serde_json::from_slice(&bytes).map_err(|source| {
                SourceDerivedNativeTargetError::InvalidRecord {
                    path: record_path.to_path_buf(),
                    source,
                }
            })?;
        validate_record(&record)?;
        validate_fixed_source(fixed_source.as_ref())?;
        let profile_fingerprint = read_profile_fingerprint(source_closure_profile.as_ref())?;
        if record.profile_fingerprint != profile_fingerprint {
            return Err(SourceDerivedNativeTargetError::ProfileFingerprintMismatch {
                expected: record.profile_fingerprint,
                actual: profile_fingerprint,
            });
        }
        let role_digest = decode_digest(&record.role_digest, "role_digest")?;
        let mut name_digests = Vec::with_capacity(record.name_digests.len());
        for name_digest in &record.name_digests {
            name_digests.push(decode_digest(name_digest, "name_digests")?);
        }
        Ok(AxTargetLocator::from_name_digests(
            role_digest,
            name_digests,
        ))
    }
}

fn validate_record(
    record: &SourceDerivedNativeTargetRecord,
) -> Result<(), SourceDerivedNativeTargetError> {
    if record.schema_version != RECORD_SCHEMA_VERSION {
        return Err(SourceDerivedNativeTargetError::RecordFieldMismatch(
            "schema_version",
        ));
    }
    if record.generated_by != GENERATED_BY {
        return Err(SourceDerivedNativeTargetError::RecordFieldMismatch(
            "generated_by",
        ));
    }
    if record.katana_revision != FIXED_KATANA_REVISION {
        return Err(SourceDerivedNativeTargetError::RecordFieldMismatch(
            "katana_revision",
        ));
    }
    for (field, value) in [
        ("profile_fingerprint", record.profile_fingerprint.as_str()),
        ("source_span_digest", record.source_span_digest.as_str()),
        ("role_digest", record.role_digest.as_str()),
    ] {
        if !is_digest(value) {
            return Err(SourceDerivedNativeTargetError::InvalidDigest(field));
        }
    }
    if record.name_digests.is_empty()
        || record
            .name_digests
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
        || record.name_digests.iter().any(|value| !is_digest(value))
    {
        return Err(SourceDerivedNativeTargetError::InvalidDigest(
            "name_digests",
        ));
    }
    Ok(())
}

fn read_profile_fingerprint(path: &Path) -> Result<String, SourceDerivedNativeTargetError> {
    let bytes =
        std::fs::read(path).map_err(|source| SourceDerivedNativeTargetError::ReadProfile {
            path: path.to_path_buf(),
            source,
        })?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|source| {
        SourceDerivedNativeTargetError::InvalidProfile {
            path: path.to_path_buf(),
            source,
        }
    })?;
    let fingerprint = value
        .get("root")
        .and_then(|root| root.get("release_profile_matrix_fingerprint"))
        .and_then(serde_json::Value::as_str)
        .filter(|value| is_digest(value))
        .ok_or_else(
            || SourceDerivedNativeTargetError::MissingProfileFingerprint {
                path: path.to_path_buf(),
            },
        )?;
    Ok(fingerprint.to_owned())
}

fn validate_fixed_source(path: &Path) -> Result<(), SourceDerivedNativeTargetError> {
    let output = run_git(path, ["rev-parse", "HEAD"])?;
    if !output.status.success() {
        return Err(SourceDerivedNativeTargetError::SourceRevisionUnavailable);
    }
    let actual = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if actual != FIXED_KATANA_REVISION {
        return Err(SourceDerivedNativeTargetError::SourceRevisionMismatch {
            expected: FIXED_KATANA_REVISION,
            actual,
        });
    }
    let output = run_git(path, ["status", "--porcelain"])?;
    if !output.status.success() {
        return Err(SourceDerivedNativeTargetError::SourceWorktreeUnavailable);
    }
    validate_worktree_status(&output.stdout)?;

    let output = run_git(path, ["symbolic-ref", "--quiet", "HEAD"])?;
    validate_detached_head(output.status.code())?;
    Ok(())
}

fn run_git<const N: usize>(
    path: &Path,
    args: [&str; N],
) -> Result<std::process::Output, SourceDerivedNativeTargetError> {
    ProcessService::create_command("git")
        .args(args)
        .current_dir(path)
        .output()
        .map_err(|_| SourceDerivedNativeTargetError::SourceGitUnavailable)
}

fn validate_worktree_status(status: &[u8]) -> Result<(), SourceDerivedNativeTargetError> {
    if status.is_empty() {
        Ok(())
    } else {
        Err(SourceDerivedNativeTargetError::SourceWorktreeDirty)
    }
}

fn validate_detached_head(
    symbolic_ref_exit_code: Option<i32>,
) -> Result<(), SourceDerivedNativeTargetError> {
    match symbolic_ref_exit_code {
        Some(1) => Ok(()),
        Some(0) => Err(SourceDerivedNativeTargetError::SourceHeadAttached),
        _ => Err(SourceDerivedNativeTargetError::SourceHeadUnavailable),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SourceDerivedNativeTargetError, SourceDerivedNativeTargetRecord, validate_detached_head,
        validate_record, validate_worktree_status,
    };

    const DIGEST: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";
    const DIGEST_ONE: &str =
        "sha256:1111111111111111111111111111111111111111111111111111111111111111";

    fn record_json() -> String {
        format!(
            r#"{{
                "schema_version":"1",
                "generated_by":"katana-source-closure-generator",
                "katana_revision":"4f6a6287c650a38633c7baeb544a92e739c68567",
                "profile_fingerprint":"{DIGEST}",
                "source_span_digest":"{DIGEST}",
                "role_digest":"{DIGEST}",
                "name_digests":["{DIGEST}"]
            }}"#
        )
    }

    #[test]
    fn record_schema_rejects_unknown_fields() {
        let json = format!(
            "{}\n",
            record_json().replace('}', ",\"raw_role\":\"AXButton\"}")
        );
        assert!(serde_json::from_str::<SourceDerivedNativeTargetRecord>(&json).is_err());
    }

    #[test]
    fn record_validation_requires_generator_marker_and_fixed_revision() {
        let result = serde_json::from_str::<SourceDerivedNativeTargetRecord>(&record_json());
        assert!(result.is_ok(), "fixture record must parse: {result:?}");
        let Some(mut record) = result.ok() else {
            return;
        };
        record.generated_by = "manual".to_owned();
        assert!(validate_record(&record).is_err());
        record.generated_by = "katana-source-closure-generator".to_owned();
        record.katana_revision = "mutable".to_owned();
        assert!(validate_record(&record).is_err());
    }

    #[test]
    fn record_validation_accepts_sorted_distinct_locale_digests() {
        let json = record_json().replace(
            &format!("[\"{DIGEST}\"]"),
            &format!("[\"{DIGEST}\",\"{DIGEST_ONE}\"]"),
        );
        let record: SourceDerivedNativeTargetRecord =
            serde_json::from_str(&json).expect("multi-locale record");
        assert!(validate_record(&record).is_ok());
    }

    #[test]
    fn record_validation_rejects_empty_or_duplicate_locale_digests() {
        let invalid_names = vec!["[]".to_string(), format!("[\"{DIGEST}\",\"{DIGEST}\"]")];
        for names in invalid_names {
            let json = record_json().replace(&format!("[\"{DIGEST}\"]"), &names);
            let record: SourceDerivedNativeTargetRecord =
                serde_json::from_str(&json).expect("record schema");
            assert!(validate_record(&record).is_err());
        }
    }

    #[test]
    fn fixed_source_validation_rejects_dirty_worktrees_without_exposing_status() {
        assert!(validate_worktree_status(b"").is_ok());
        let result = validate_worktree_status(b" M secret-source.rs\n");
        assert!(matches!(
            result,
            Err(SourceDerivedNativeTargetError::SourceWorktreeDirty)
        ));
        let error = SourceDerivedNativeTargetError::SourceWorktreeDirty;
        assert!(!error.to_string().contains("secret-source.rs"));
    }

    #[test]
    fn fixed_source_validation_requires_detached_head() {
        assert!(matches!(
            validate_detached_head(Some(0)),
            Err(SourceDerivedNativeTargetError::SourceHeadAttached)
        ));
        assert!(validate_detached_head(Some(1)).is_ok());
        assert!(matches!(
            validate_detached_head(Some(128)),
            Err(SourceDerivedNativeTargetError::SourceHeadUnavailable)
        ));
    }
}
