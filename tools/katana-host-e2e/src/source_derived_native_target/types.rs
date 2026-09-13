use serde::Deserialize;
use std::fmt;
use std::path::PathBuf;

pub struct SourceDerivedNativeTarget;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SourceDerivedNativeTargetRecord {
    pub(super) schema_version: String,
    pub(super) generated_by: String,
    pub(super) katana_revision: String,
    pub(super) profile_fingerprint: String,
    pub(super) source_span_digest: String,
    pub(super) role_digest: String,
    pub(super) name_digests: Vec<String>,
}

#[derive(Debug)]
pub enum SourceDerivedNativeTargetError {
    ReadRecord {
        path: PathBuf,
        source: std::io::Error,
    },
    InvalidRecord {
        path: PathBuf,
        source: serde_json::Error,
    },
    RecordFieldMismatch(&'static str),
    InvalidDigest(&'static str),
    ReadProfile {
        path: PathBuf,
        source: std::io::Error,
    },
    InvalidProfile {
        path: PathBuf,
        source: serde_json::Error,
    },
    MissingProfileFingerprint {
        path: PathBuf,
    },
    ProfileFingerprintMismatch {
        expected: String,
        actual: String,
    },
    SourceGitUnavailable,
    SourceRevisionUnavailable,
    SourceRevisionMismatch {
        expected: &'static str,
        actual: String,
    },
    SourceWorktreeUnavailable,
    SourceWorktreeDirty,
    SourceHeadAttached,
    SourceHeadUnavailable,
}

impl fmt::Display for SourceDerivedNativeTargetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadRecord { .. } => write!(formatter, "cannot read generated target record"),
            Self::InvalidRecord { .. } => write!(formatter, "invalid generated target record JSON"),
            Self::RecordFieldMismatch(field) => write!(
                formatter,
                "generated target record field {field} is invalid"
            ),
            Self::InvalidDigest(field) => write!(
                formatter,
                "generated target record field {field} is not a SHA-256 digest"
            ),
            Self::ReadProfile { .. } => write!(formatter, "cannot read source-closure profile"),
            Self::InvalidProfile { .. } => write!(formatter, "invalid source-closure profile JSON"),
            Self::MissingProfileFingerprint { .. } => {
                write!(
                    formatter,
                    "source-closure profile has no release profile fingerprint"
                )
            }
            Self::ProfileFingerprintMismatch { expected, actual } => write!(
                formatter,
                "profile fingerprint {actual} does not match record {expected}"
            ),
            Self::SourceGitUnavailable => write!(formatter, "cannot invoke fixed KatanA Git check"),
            Self::SourceRevisionUnavailable => {
                write!(formatter, "cannot verify fixed KatanA revision")
            }
            Self::SourceRevisionMismatch { expected, actual } => write!(
                formatter,
                "fixed KatanA revision {actual} does not match {expected}"
            ),
            Self::SourceWorktreeUnavailable => {
                write!(formatter, "cannot verify fixed KatanA worktree state")
            }
            Self::SourceWorktreeDirty => write!(formatter, "fixed KatanA worktree is not clean"),
            Self::SourceHeadAttached => write!(formatter, "fixed KatanA HEAD is not detached"),
            Self::SourceHeadUnavailable => {
                write!(formatter, "cannot verify fixed KatanA HEAD state")
            }
        }
    }
}

impl std::error::Error for SourceDerivedNativeTargetError {}

#[cfg(test)]
mod tests {
    use super::SourceDerivedNativeTargetError;
    use std::path::PathBuf;

    #[test]
    fn errors_do_not_display_raw_paths() {
        let error = SourceDerivedNativeTargetError::MissingProfileFingerprint {
            path: PathBuf::from("/private/source-closure-profile.json"),
        };
        assert!(
            !error
                .to_string()
                .contains("/private/source-closure-profile.json")
        );
    }
}
