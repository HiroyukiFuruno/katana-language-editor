use serde::Serialize;
use std::fmt;

pub(crate) const DIGEST_BYTES: usize = 32;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RedactedNode {
    pub(crate) role: String,
    pub(crate) title: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) enabled: Option<bool>,
    pub(crate) focused: Option<bool>,
    pub(crate) read_only: Option<bool>,
    pub(crate) ancestor_roles: Vec<String>,
    pub(crate) depth: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NativeAxWorkspaceObservation {
    pub schema: &'static str,
    pub katana_revision: &'static str,
    pub source_span_digest: [u8; DIGEST_BYTES],
    pub process_identity_digest: [u8; DIGEST_BYTES],
    pub observation_generation: u64,
    pub workspace_basename_digest: [u8; DIGEST_BYTES],
    pub workspace_correlation_candidates: usize,
    pub editor_candidates: Vec<NativeAxEditorCandidate>,
    pub canonical_digest: [u8; DIGEST_BYTES],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NativeAxEditorCandidate {
    pub role_digest: [u8; DIGEST_BYTES],
    pub title_digest: Option<[u8; DIGEST_BYTES]>,
    pub description_digest: Option<[u8; DIGEST_BYTES]>,
    pub enabled: bool,
    pub focused: bool,
    pub read_only: bool,
    pub ancestor_role_digests: Vec<[u8; DIGEST_BYTES]>,
    pub depth: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeAxSourceContract {
    pub source_span_digest: [u8; DIGEST_BYTES],
    pub accesskit_role_digest: [u8; DIGEST_BYTES],
    pub native_role: NativeAxRole,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeAxRole {
    MultilineTextInput,
}

impl NativeAxRole {
    pub(crate) fn from_accesskit_role(role: &str) -> Result<Self, NativeAxObservationError> {
        match role {
            "MultilineTextInput" => Ok(Self::MultilineTextInput),
            _ => Err(NativeAxObservationError::UnsupportedRoleTranslation),
        }
    }

    pub(crate) fn native_ax_role(self) -> &'static str {
        match self {
            Self::MultilineTextInput => "AXTextArea",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeAxObservationError {
    UnsupportedPlatform,
    AccessibilityUnavailable,
    AttributeMissing,
    AttributeTypeMismatch,
    UnsupportedRoleTranslation,
    NullAttribute,
    TraversalCycle,
    TraversalDepthExceeded,
    TraversalNodeLimitExceeded,
    WorkspaceMismatch,
    WorkspaceDuplicate,
    EditorTargetMissing,
    EditorTargetAmbiguous,
}

impl fmt::Display for NativeAxObservationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedPlatform => "native AX observation is unsupported on this platform",
            Self::AccessibilityUnavailable => "native AX observation is unavailable",
            Self::AttributeMissing => "required native AX attribute is missing",
            Self::AttributeTypeMismatch => "native AX attribute has an unexpected type",
            Self::UnsupportedRoleTranslation => "AccessKit role has no supported native AX translation",
            Self::NullAttribute => "native AX returned a null attribute",
            Self::TraversalCycle => "native AX traversal encountered a cycle",
            Self::TraversalDepthExceeded => "native AX traversal exceeded its depth bound",
            Self::TraversalNodeLimitExceeded => "native AX traversal exceeded its node bound",
            Self::WorkspaceMismatch => "native AX workspace correlation did not match",
            Self::WorkspaceDuplicate => "native AX workspace correlation was duplicated",
            Self::EditorTargetMissing => "native AX frame has no editor candidate",
            Self::EditorTargetAmbiguous => "native AX frame has multiple editor candidates",
        })
    }
}

impl std::error::Error for NativeAxObservationError {}
