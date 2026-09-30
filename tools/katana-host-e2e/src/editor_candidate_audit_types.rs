use std::fmt;

const DIGEST_BYTES: usize = 32;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditorSourceContract {
    pub katana_revision: String,
    pub source_span_digest: String,
    pub role: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EditorCandidateState {
    pub disabled: bool,
    pub hidden: bool,
    pub read_only: bool,
    pub focused: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditorCandidateFacts {
    pub role_digest: [u8; DIGEST_BYTES],
    pub name_digest: [u8; DIGEST_BYTES],
    pub description_digest: Option<[u8; DIGEST_BYTES]>,
    pub ancestor_role_digests: Vec<[u8; DIGEST_BYTES]>,
    pub state: EditorCandidateState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditorCandidateObservation {
    pub frame_generation: u64,
    pub source_span_digest: [u8; DIGEST_BYTES],
    pub candidate: EditorCandidateFacts,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EditorCandidateAuditError {
    TargetMissing,
    TargetAmbiguous,
    UnsupportedEditorIdentity,
}

impl fmt::Display for EditorCandidateAuditError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::TargetMissing => "the current frame has no editor candidate",
            Self::TargetAmbiguous => "the current frame has multiple editor candidates",
            Self::UnsupportedEditorIdentity => "the fixed editor identity is unsupported",
        })
    }
}

impl std::error::Error for EditorCandidateAuditError {}
