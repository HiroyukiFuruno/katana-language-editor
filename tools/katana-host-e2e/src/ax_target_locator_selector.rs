use super::types::{AxTargetLocator, AxTargetProof, AxTargetSelectionError, DIGEST_BYTES};
use sha2::{Digest, Sha256};

const PROOF_DOMAIN: &[u8] = b"katana-ax-target-v1";

#[derive(Clone, Copy)]
pub(crate) struct Candidate {
    pub(crate) role_digest: [u8; DIGEST_BYTES],
    pub(crate) name_digest: [u8; DIGEST_BYTES],
}

pub(crate) struct CandidateDigest;

impl CandidateDigest {
    pub(crate) fn of(value: &str) -> [u8; DIGEST_BYTES] {
        Sha256::digest(value.as_bytes()).into()
    }
}

pub(crate) struct CandidateSelector;

impl CandidateSelector {
    pub(crate) fn select(
        locator: &AxTargetLocator,
        candidates: &[Candidate],
    ) -> Result<usize, AxTargetSelectionError> {
        let mut selected = None;
        for (index, candidate) in candidates.iter().enumerate() {
            if candidate.role_digest == locator.role_digest
                && locator.name_digests.contains(&candidate.name_digest)
                && selected.replace(index).is_some()
            {
                return Err(AxTargetSelectionError::TargetAmbiguous);
            }
        }
        selected.ok_or(AxTargetSelectionError::TargetMissing)
    }

    pub(crate) fn proof(
        locator: &AxTargetLocator,
        candidates: &[Candidate],
        index: usize,
    ) -> AxTargetProof {
        let mut snapshot = Sha256::new();
        for candidate in candidates {
            snapshot.update(candidate.role_digest);
            snapshot.update(candidate.name_digest);
        }
        let snapshot_digest: [u8; DIGEST_BYTES] = snapshot.finalize().into();
        let mut token = Sha256::new();
        token.update(PROOF_DOMAIN);
        token.update(snapshot_digest);
        token.update((index as u64).to_be_bytes());
        AxTargetProof {
            role_digest: locator.role_digest,
            name_digests: locator.name_digests.clone(),
            snapshot_target_token: token.finalize().into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Candidate, CandidateDigest, CandidateSelector};
    use crate::ax_target_locator::types::{AxTargetLocator, AxTargetSelectionError};

    fn candidate(role: &str, name: &str) -> Candidate {
        Candidate {
            role_digest: CandidateDigest::of(role),
            name_digest: CandidateDigest::of(name),
        }
    }

    #[test]
    fn selector_rejects_missing_candidate() {
        let locator = AxTargetLocator::from_accessible_name("AXButton", "Run");
        assert_eq!(
            CandidateSelector::select(&locator, &[candidate("AXButton", "Stop")]),
            Err(AxTargetSelectionError::TargetMissing)
        );
    }

    #[test]
    fn selector_rejects_ambiguous_candidate() {
        let locator = AxTargetLocator::from_accessible_name("AXButton", "Run");
        assert_eq!(
            CandidateSelector::select(
                &locator,
                &[candidate("AXButton", "Run"), candidate("AXButton", "Run")]
            ),
            Err(AxTargetSelectionError::TargetAmbiguous)
        );
    }

    #[test]
    fn selector_accepts_any_allowed_locale_name_digest() {
        let locator = AxTargetLocator::from_name_digests(
            CandidateDigest::of("AXButton"),
            vec![
                CandidateDigest::of("Open Workspace"),
                CandidateDigest::of("Arbeitsbereich öffnen"),
            ],
        );
        assert_eq!(
            CandidateSelector::select(&locator, &[candidate("AXButton", "Arbeitsbereich öffnen")]),
            Ok(0)
        );
    }
}
