#[derive(Clone, Eq, PartialEq)]
pub struct HostTargetLocator {
    pub role: Role,
    pub accessible_name_sha256: [u8; SHA256_DIGEST_BYTES],
    accessible_name_candidates: Vec<[u8; SHA256_DIGEST_BYTES]>,
    requires_menu_ancestor: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostTargetSelectionError {
    InvalidDescriptor,
    MissingTarget,
    AmbiguousTarget,
    AccessibleNameMissing,
    AccessibleNameMismatch,
    BoundsMissing,
    DisabledTarget,
    MenuAncestorMissing,
}

impl fmt::Display for HostTargetSelectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidDescriptor => {
                "the context-menu descriptor contains an invalid label digest"
            }
            Self::MissingTarget => "no AccessKit target matched the locator role",
            Self::AmbiguousTarget => "multiple AccessKit targets matched the locator",
            Self::AccessibleNameMissing => "the AccessKit target has no accessible name",
            Self::AccessibleNameMismatch => "the AccessKit accessible name hash did not match",
            Self::BoundsMissing => "the AccessKit target has no current bounded geometry",
            Self::DisabledTarget => "the AccessKit target is disabled",
            Self::MenuAncestorMissing => "the AccessKit target has no current Menu ancestor",
        })
    }
}
impl std::error::Error for HostTargetSelectionError {}

#[derive(Debug, PartialEq)]
pub(crate) struct CurrentHostTarget {
    pub(crate) frame_hash: [u8; SHA256_DIGEST_BYTES],
    pub(crate) node_id: NodeId,
    pub(crate) bounds: accesskit::Rect,
}
