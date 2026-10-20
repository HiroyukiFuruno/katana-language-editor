use super::super::super::artifact_model::LeafStatusRecord;
use super::super::super::consumer_artifact_intake::{EvidenceStage, ValidatedConsumerArtifact};
#[path = "consumer_join_records_output.rs"]
mod output;
#[cfg(test)]
#[path = "consumer_join_records_tests.rs"]
mod tests;
use std::collections::{BTreeMap, BTreeSet};

struct ConsumerStage {
    stage_id: String,
    interaction: String,
    media_sha256: String,
    root_record_hash: String,
}
pub(super) struct ConsumerJoinRecords {
    leafs: Vec<LeafStatusRecord>,
    profiles: BTreeSet<String>,
    receipts: BTreeSet<String>,
    stages: BTreeMap<(String, String), ConsumerStage>,
}
impl ConsumerJoinRecords {
    pub(super) fn new(leafs: Vec<LeafStatusRecord>) -> Self {
        Self {
            leafs,
            profiles: BTreeSet::new(),
            receipts: BTreeSet::new(),
            stages: BTreeMap::new(),
        }
    }

    pub(super) fn add_profile(
        &mut self,
        profile_id: &str,
        artifact: &ValidatedConsumerArtifact,
    ) -> Result<(), String> {
        if artifact.profile_id() != profile_id {
            return Err("consumer artifact profile differs from requested profile".into());
        }
        if self.profiles.contains(profile_id) {
            return Err(format!("consumer profile was added twice: {profile_id}"));
        }
        let mut pending = BTreeMap::new();
        let stages: &[EvidenceStage] = artifact.stages();
        for stage in stages {
            let leaf = self
                .leafs
                .iter()
                .find(|leaf| leaf.leaf_id == stage.leaf_id)
                .ok_or_else(|| format!("unknown consumer leaf: {}", stage.leaf_id))?;
            if !leaf.required_profile_ids.iter().any(|id| id == profile_id) {
                return Err(format!("unrequired consumer profile: {profile_id}"));
            }
            let key = (stage.leaf_id.clone(), profile_id.to_owned());
            if self.stages.contains_key(&key) || pending.contains_key(&key) {
                return Err(format!("duplicate consumer leaf/profile: {key:?}"));
            }
            if self
                .stages
                .iter()
                .chain(pending.iter())
                .any(|((leaf_id, _), previous)| {
                    leaf_id == &stage.leaf_id && previous.stage_id != stage.stage_id
                })
            {
                return Err(format!("consumer stage differs for leaf {}", stage.leaf_id));
            }
            pending.insert(
                key,
                ConsumerStage {
                    stage_id: stage.stage_id.clone(),
                    interaction: stage.interaction.clone(),
                    media_sha256: stage.media_sha256.clone(),
                    root_record_hash: stage.root_record_hash.clone(),
                },
            );
        }
        if !artifact.receipt_fingerprints().is_disjoint(&self.receipts) {
            return Err("consumer receipt fingerprint is reused across profiles".into());
        }
        self.profiles.insert(profile_id.to_owned());
        self.receipts
            .extend(artifact.receipt_fingerprints().iter().cloned());
        self.stages.extend(pending);
        Ok(())
    }
}
