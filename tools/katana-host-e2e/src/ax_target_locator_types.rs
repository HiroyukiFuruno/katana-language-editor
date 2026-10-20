use std::fmt;

#[cfg(test)]
use crate::ax_target_locator::selector::CandidateDigest;

pub(crate) const DIGEST_BYTES: usize = 32;

#[derive(Clone, Eq, PartialEq)]
pub struct AxTargetLocator {
    pub(crate) role_digest: [u8; DIGEST_BYTES],
    pub(crate) name_digests: Vec<[u8; DIGEST_BYTES]>,
}

impl AxTargetLocator {
    pub fn from_digests(role_digest: [u8; DIGEST_BYTES], name_digest: [u8; DIGEST_BYTES]) -> Self {
        Self::from_name_digests(role_digest, vec![name_digest])
    }

    pub(crate) fn from_name_digests(
        role_digest: [u8; DIGEST_BYTES],
        name_digests: Vec<[u8; DIGEST_BYTES]>,
    ) -> Self {
        Self {
            role_digest,
            name_digests,
        }
    }

    #[cfg(test)]
    pub(crate) fn from_accessible_name(role: &str, name: &str) -> Self {
        Self::from_digests(CandidateDigest::of(role), CandidateDigest::of(name))
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct AxTargetProof {
    pub(crate) role_digest: [u8; DIGEST_BYTES],
    pub(crate) name_digests: Vec<[u8; DIGEST_BYTES]>,
    pub(crate) snapshot_target_token: [u8; DIGEST_BYTES],
}

impl fmt::Debug for AxTargetLocator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AxTargetLocator")
            .field("role_digest", &self.role_digest)
            .field("name_digests", &self.name_digests)
            .finish()
    }
}

impl fmt::Debug for AxTargetProof {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AxTargetProof")
            .field("role_digest", &self.role_digest)
            .field("name_digests", &self.name_digests)
            .field("snapshot_target_token", &self.snapshot_target_token)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AxTargetSelectionError {
    UnsupportedPlatform,
    AttributeMissing,
    AttributeTypeMismatch,
    NullAttribute,
    AccessibilityFailure,
    TraversalAmbiguous,
    TraversalCycle,
    TraversalDepthExceeded,
    TargetMissing,
    TargetAmbiguous,
}

impl fmt::Display for AxTargetSelectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedPlatform => "native AX target lookup is unsupported on this platform",
            Self::AttributeMissing => "required AX attribute is missing",
            Self::AttributeTypeMismatch => "AX attribute has an unexpected type",
            Self::NullAttribute => "AX returned a null attribute value",
            Self::AccessibilityFailure => "AX rejected an attribute request",
            Self::TraversalAmbiguous => "AX traversal returned an ambiguous child collection",
            Self::TraversalCycle => "AX traversal encountered a cycle",
            Self::TraversalDepthExceeded => "AX traversal exceeded its depth limit",
            Self::TargetMissing => "no AX target matched the locator",
            Self::TargetAmbiguous => "multiple AX targets matched the locator",
        })
    }
}

impl std::error::Error for AxTargetSelectionError {}

#[cfg(test)]
mod tests {
    use super::{AxTargetLocator, DIGEST_BYTES};

    #[test]
    fn locator_debug_contains_digests_but_not_raw_inputs() {
        let locator = AxTargetLocator::from_accessible_name("AXButton", "Never-export-this");
        let rendered = format!("{locator:?}");
        assert!(!rendered.contains("AXButton"));
        assert!(!rendered.contains("Never-export-this"));
        assert_ne!(locator.role_digest, [0; DIGEST_BYTES]);
        assert!(
            locator
                .name_digests
                .iter()
                .all(|digest| *digest != [0; DIGEST_BYTES])
        );
    }
}
