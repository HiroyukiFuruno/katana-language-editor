use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use super::super::artifact_model::{
    KATANA_ADOPTION_ISSUE_URL, KatanaHostE2eRequirement, KatanaHostE2eState, LeafStatusRecord,
    StorybookArtifactEntry,
};
use super::super::consumer_artifact_intake::ConsumerArtifactIntake;
use super::super::execution_record_model::{ExecutionKind, ExecutionRecord};
use super::super::fingerprint::sha256_hex;
use super::super::generic_interaction_mapping::{
    ConsumerPlanProvenance, GenericInteractionMapping,
};
use super::model::{Bundle, ProfileEvidence};
use super::paths::{safe_dir, safe_file};
use super::profiles::canonical_required_profile_ids;
use crate::source_closure::root_model::CANONICAL_PROFILE_IDS;

const EVIDENCE_FILE: &str = "kle-consumer-artifact-evidence.json";

struct ConsumerStageSnapshot {
    leaf_id: String,
    interaction: String,
    stage_id: String,
    media_sha256: String,
    root_record_hash: String,
}

pub(super) struct Reconstruction {
    pub(super) leafs: Vec<LeafStatusRecord>,
    pub(super) executions: Vec<ExecutionRecord>,
    pub(super) storybook: Vec<StorybookArtifactEntry>,
}

pub(super) fn reconstruct(
    root: &Path,
    loaded: &super::super::load::LoadedArtifacts,
    mapping: &GenericInteractionMapping,
    bundle: &Bundle,
    source_hash: &str,
    profiles: &BTreeMap<&str, &ProfileEvidence>,
    mut leafs: Vec<LeafStatusRecord>,
) -> Result<Reconstruction, String> {
    leafs.sort_by(|left, right| left.leaf_id.cmp(&right.leaf_id));
    let mut stages = BTreeMap::new();
    let mut receipts = BTreeSet::new();
    for profile_id in CANONICAL_PROFILE_IDS {
        let profile = profiles
            .get(profile_id)
            .ok_or_else(|| format!("bundle is missing profile: {profile_id}"))?;
        let plan = mapping.consumer_plan(
            &loaded.leaf_manifest.canonical_binding,
            &loaded.branch_catalog,
            profile_id,
            ConsumerPlanProvenance {
                run_id: bundle.run_id.clone(),
                source_closure_sha256: source_hash.to_owned(),
                profile_fingerprint: profile.profile_fingerprint.clone(),
            },
        )?;
        let plan_bytes = serde_json::to_vec(&plan)
            .map_err(|error| format!("failed to serialize consumer plan: {error}"))?;
        if sha256_hex(&plan_bytes) != profile.plan_sha256 {
            return Err(format!(
                "consumer plan hash differs for profile: {profile_id}"
            ));
        }
        let profile_dir = safe_dir(root, &format!("consumer/{profile_id}"))?;
        let evidence_path = safe_file(&profile_dir, EVIDENCE_FILE, "consumer evidence")?;
        let evidence_bytes = fs::read(&evidence_path)
            .map_err(|error| format!("consumer evidence cannot be read: {error}"))?;
        if sha256_hex(&evidence_bytes) != profile.evidence_sha256 {
            return Err(format!(
                "consumer evidence hash differs for profile: {profile_id}"
            ));
        }
        let validated =
            ConsumerArtifactIntake::validate_profile_artifact(&plan_bytes, &profile_dir)?;
        register_receipts(&mut receipts, validated.receipt_fingerprints())?;
        for stage in validated.stages() {
            let key = (stage.leaf_id.clone(), profile_id.to_owned());
            let snapshot = ConsumerStageSnapshot {
                leaf_id: stage.leaf_id.clone(),
                interaction: stage.interaction.clone(),
                stage_id: stage.stage_id.clone(),
                media_sha256: stage.media_sha256.clone(),
                root_record_hash: stage.root_record_hash.clone(),
            };
            if stages.insert(key.clone(), snapshot).is_some() {
                return Err(format!("duplicate consumer stage: {key:?}"));
            }
        }
    }

    let mut executions = Vec::new();
    let mut storybook = Vec::new();
    for leaf in &mut leafs {
        let required = canonical_required_profile_ids(&leaf.required_profile_ids)?;
        let mut stage_id = None;
        for profile_id in required {
            let stage = stages
                .get(&(leaf.leaf_id.clone(), profile_id.to_owned()))
                .ok_or_else(|| {
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
                execution_profile_id: profile_id.to_owned(),
                leaf_id: stage.leaf_id.clone(),
                action_route_signature: format!("kuc:{}", stage.interaction),
                status: "passing".into(),
            });
            storybook.push(StorybookArtifactEntry {
                story_id: format!("consumer:{profile_id}:{}", stage.stage_id),
                leaf_id: stage.leaf_id.clone(),
                stage_id: stage.stage_id.clone(),
                frame_record_id: stage.root_record_hash.clone(),
                media_path: format!("consumer/{profile_id}/{}.png", stage.stage_id),
                media_sha256: format!("sha256:{}", stage.media_sha256),
                profile_id: profile_id.to_owned(),
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
    executions.sort_by(|left, right| {
        left.leaf_id
            .cmp(&right.leaf_id)
            .then_with(|| left.execution_profile_id.cmp(&right.execution_profile_id))
    });
    storybook.sort_by(|left, right| {
        left.leaf_id
            .cmp(&right.leaf_id)
            .then_with(|| left.profile_id.cmp(&right.profile_id))
    });
    Ok(Reconstruction {
        leafs,
        executions,
        storybook,
    })
}

fn register_receipts(
    used: &mut BTreeSet<String>,
    current: &BTreeSet<String>,
) -> Result<(), String> {
    if !used.is_disjoint(current) {
        return Err("consumer release evidence reuses a receipt across profiles".into());
    }
    used.extend(current.iter().cloned());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{BTreeSet, register_receipts};

    #[test]
    fn cross_profile_receipt_reuse_is_rejected_without_partial_registration() -> Result<(), String>
    {
        let mut used = BTreeSet::from(["first".into()]);
        let reused = BTreeSet::from(["first".into(), "new".into()]);
        assert!(register_receipts(&mut used, &reused).is_err());
        assert_eq!(used, BTreeSet::from(["first".into()]));
        register_receipts(&mut used, &BTreeSet::from(["second".into()]))?;
        assert_eq!(used.len(), 2);
        Ok(())
    }
}
