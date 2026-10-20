use super::super::root_model::CANONICAL_PROFILE_IDS;
use super::files;
use super::model::{
    EVIDENCE_FILE, EVIDENCE_SCHEMA, Evidence, EvidenceStage, INITIAL_ROOT_REVISION,
    KUC_STAGE_PREFIX, PLAN_SCHEMA, Plan,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

const KUC_INTERACTIONS: [&str; 10] = [
    "text_input",
    "ime_commit",
    "selection",
    "scroll",
    "toolbar_activation",
    "floating_toolbar",
    "search",
    "context_menu",
    "accessibility_activation",
    "viewport_resize",
];

pub(super) fn validate_profile_artifact(
    plan_bytes: &[u8],
    artifact_dir: &Path,
) -> Result<(), String> {
    let plan: Plan = serde_json::from_slice(plan_bytes)
        .map_err(|error| format!("consumer artifact plan is invalid: {error}"))?;
    validate_plan(&plan)?;
    let root = files::safe_root(artifact_dir)?;
    let evidence: Evidence = files::read_json(&root, EVIDENCE_FILE, "consumer artifact evidence")?;
    validate_evidence_header(&plan, &evidence)?;
    let stages = index_evidence(&plan, &evidence)?;
    let mut receipts = BTreeSet::new();
    for stage in stages.values() {
        files::validate_stage(&root, stage, &mut receipts)?;
    }
    Ok(())
}

fn validate_plan(plan: &Plan) -> Result<(), String> {
    if plan.schema != PLAN_SCHEMA {
        return Err(format!(
            "unsupported consumer artifact plan schema: {}",
            plan.schema
        ));
    }
    validate_provenance(
        &plan.run_id,
        &plan.source_closure_sha256,
        &plan.profile_fingerprint,
    )?;
    if !CANONICAL_PROFILE_IDS.contains(&plan.profile_id.as_str()) {
        return Err(format!(
            "consumer artifact profile is not canonical: {}",
            plan.profile_id
        ));
    }
    if plan.stages.is_empty() {
        return Err("consumer artifact plan has no stages".into());
    }
    let mut leaves = BTreeSet::new();
    for stage in &plan.stages {
        if stage.leaf_id.trim().is_empty()
            || stage.interaction.trim().is_empty()
            || !KUC_INTERACTIONS.contains(&stage.interaction.as_str())
        {
            return Err("consumer artifact plan contains an invalid stage".into());
        }
        if !leaves.insert(&stage.leaf_id) {
            return Err(format!(
                "consumer artifact plan has duplicate leaf: {}",
                stage.leaf_id
            ));
        }
    }
    Ok(())
}

fn validate_evidence_header(plan: &Plan, evidence: &Evidence) -> Result<(), String> {
    if evidence.schema != EVIDENCE_SCHEMA {
        return Err(format!(
            "unsupported consumer artifact evidence schema: {}",
            evidence.schema
        ));
    }
    if plan.run_id != evidence.run_id
        || plan.source_closure_sha256 != evidence.source_closure_sha256
        || plan.profile_fingerprint != evidence.profile_fingerprint
        || plan.profile_id != evidence.profile_id
    {
        return Err("consumer artifact evidence provenance differs from the plan".into());
    }
    if evidence.stages.is_empty() {
        return Err("consumer artifact evidence has no stages".into());
    }
    Ok(())
}

fn index_evidence<'a>(
    plan: &Plan,
    evidence: &'a Evidence,
) -> Result<BTreeMap<&'a str, &'a EvidenceStage>, String> {
    let expected = plan
        .stages
        .iter()
        .map(|stage| (stage.leaf_id.as_str(), stage.interaction.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut actual = BTreeMap::new();
    let mut stage_ids = BTreeSet::new();
    let mut media_paths = BTreeSet::new();
    for (index, stage) in evidence.stages.iter().enumerate() {
        let expected_stage = format!("{KUC_STAGE_PREFIX}{index:04}");
        let expected_revision = INITIAL_ROOT_REVISION + index as u64;
        if stage.stage_id != expected_stage || stage.root_revision != expected_revision {
            return Err(format!(
                "consumer artifact stage order or identity differs at index {index}"
            ));
        }
        let planned = plan
            .stages
            .get(index)
            .ok_or_else(|| "consumer artifact evidence has extra stages".to_string())?;
        if stage.leaf_id != planned.leaf_id || stage.interaction != planned.interaction {
            return Err(format!(
                "consumer artifact stage order differs at index {index}"
            ));
        }
        let Some(expected_interaction) = expected.get(stage.leaf_id.as_str()) else {
            return Err(format!(
                "consumer artifact evidence has unknown leaf: {}",
                stage.leaf_id
            ));
        };
        if *expected_interaction != stage.interaction
            || !KUC_INTERACTIONS.contains(&stage.interaction.as_str())
        {
            return Err(format!(
                "consumer artifact interaction differs for leaf: {}",
                stage.leaf_id
            ));
        }
        if stage.leaf_id.trim().is_empty() || stage.stage_id.trim().is_empty() {
            return Err("consumer artifact evidence contains an empty stage field".into());
        }
        let expected_media = format!("{}.png", stage.stage_id);
        if stage.media_path != expected_media {
            return Err(format!(
                "consumer artifact media path differs for stage: {}",
                stage.stage_id
            ));
        }
        if actual.insert(stage.leaf_id.as_str(), stage).is_some()
            || !stage_ids.insert(stage.stage_id.as_str())
            || !media_paths.insert(stage.media_path.as_str())
        {
            return Err("consumer artifact evidence has duplicate leaf, stage, or media".into());
        }
    }
    if actual.len() != expected.len() {
        return Err("consumer artifact evidence does not cover the plan one-to-one".into());
    }
    Ok(actual)
}

fn validate_provenance(run_id: &str, source: &str, profile: &str) -> Result<(), String> {
    if run_id.trim().is_empty()
        || run_id.chars().any(char::is_whitespace)
        || !files::prefixed_sha256(source)
        || !files::prefixed_sha256(profile)
    {
        return Err("consumer artifact provenance is invalid".into());
    }
    Ok(())
}
