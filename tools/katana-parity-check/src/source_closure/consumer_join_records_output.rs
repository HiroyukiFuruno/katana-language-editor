use super::super::super::super::artifact_model::{
    KATANA_ADOPTION_ISSUE_URL, KatanaHostE2eRequirement, KatanaHostE2eState, LeafManifestArtifact,
    StorybookArtifactEntry,
};
use super::super::super::super::execution_record_model::{
    ExecutionKind, ExecutionRecord, ExecutionRecordArtifact,
};
use super::super::write_json;
use super::{ConsumerJoinRecords, ConsumerStage};
use std::path::Path;

impl ConsumerJoinRecords {
    pub(in crate::source_closure::operational_generic_mapping::consumer_join) fn write(
        &self,
        staging: &Path,
        manifest: &LeafManifestArtifact,
    ) -> Result<(), String> {
        let mut leafs = self.leafs.clone();
        let mut executions = Vec::new();
        let mut stories = Vec::new();
        for leaf in &mut leafs {
            let required_profiles = leaf.required_profile_ids.clone();
            if required_profiles.is_empty() {
                return Err(format!(
                    "leaf has no required consumer profiles: {}",
                    leaf.leaf_id
                ));
            }
            let mut stage_id = None;
            for profile_id in required_profiles {
                let key = (leaf.leaf_id.clone(), profile_id.clone());
                let stage: &ConsumerStage = self.stages.get(&key).ok_or_else(|| {
                    format!(
                        "consumer evidence is missing leaf {} for profile {profile_id}",
                        leaf.leaf_id
                    )
                })?;
                if stage_id
                    .as_ref()
                    .is_some_and(|expected| expected != &stage.stage_id)
                {
                    return Err(format!(
                        "consumer stage differs across profiles for leaf {}",
                        leaf.leaf_id
                    ));
                }
                stage_id.get_or_insert_with(|| stage.stage_id.clone());
                executions.push(ExecutionRecord {
                    execution_kind: ExecutionKind::KucConsumerArtifact,
                    execution_id: format!("kuc-consumer:{profile_id}:{}", stage.stage_id),
                    execution_profile_id: profile_id.clone(),
                    leaf_id: leaf.leaf_id.clone(),
                    action_route_signature: format!("kuc:{}", stage.interaction),
                    status: "passing".into(),
                });
                stories.push(StorybookArtifactEntry {
                    story_id: format!("consumer:{profile_id}:{}", stage.stage_id),
                    leaf_id: leaf.leaf_id.clone(),
                    stage_id: stage.stage_id.clone(),
                    frame_record_id: stage.root_record_hash.clone(),
                    media_path: format!("consumer/{profile_id}/{}.png", stage.stage_id),
                    media_sha256: format!("sha256:{}", stage.media_sha256),
                    profile_id,
                });
            }
            leaf.host_e2e = KatanaHostE2eRequirement {
                state: KatanaHostE2eState::DownstreamRequired,
                adoption_issue_url: KATANA_ADOPTION_ISSUE_URL.to_owned(),
            };
            leaf.execution_id = None;
            leaf.storybook_stage_id =
                stage_id.ok_or_else(|| format!("leaf has no consumer stage: {}", leaf.leaf_id))?;
            leaf.status = "passing".into();
        }
        if self.stages.len() != executions.len() {
            return Err(
                "consumer evidence contains unexpected or duplicate leaf/profile records".into(),
            );
        }
        leafs.sort_by(|left, right| left.leaf_id.cmp(&right.leaf_id));
        executions.sort_by(|left, right| {
            (&left.leaf_id, &left.execution_profile_id)
                .cmp(&(&right.leaf_id, &right.execution_profile_id))
        });
        stories.sort_by(|left, right| {
            (&left.leaf_id, &left.profile_id).cmp(&(&right.leaf_id, &right.profile_id))
        });
        let mut leaf_manifest = manifest.clone();
        leaf_manifest.leafs = leafs;
        write_json(
            &staging.join("leaf-manifest.json"),
            &leaf_manifest,
            "leaf manifest",
        )?;
        write_json(
            &staging.join("execution-record.json"),
            &ExecutionRecordArtifact {
                root: manifest.root.clone(),
                executions,
                stale_or_missing_leafs: Vec::new(),
            },
            "execution record",
        )?;
        write_json(
            &staging.join("storybook-artifacts.json"),
            &super::super::super::super::artifact_model::StorybookArtifacts {
                root: manifest.root.clone(),
                artifacts: stories,
            },
            "storybook artifacts",
        )
    }
}
