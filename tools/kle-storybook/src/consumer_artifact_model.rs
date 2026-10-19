use katana_ui_core::egui::text_command_surface::GenericInteractionClass;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub(super) const PLAN_SCHEMA: &str = "kle.consumer-artifact-plan.v1";
pub(super) const EVIDENCE_SCHEMA: &str = "kle-consumer-artifact-evidence.v1";
pub(super) const EVIDENCE_MANIFEST: &str = "kle-consumer-artifact-evidence.json";
pub(super) type ValidatedPlan = (String, Vec<ConsumerArtifactStageRequest>);

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SourceLeafArtifactPlan {
    pub(super) schema: String,
    pub(super) profile_id: String,
    pub(super) stages: Vec<SourceLeafArtifactStage>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SourceLeafArtifactStage {
    pub(super) leaf_id: String,
    pub(super) interaction: String,
}

#[derive(Debug, Serialize)]
pub(super) struct SourceLeafArtifactEvidence {
    pub(super) schema: &'static str,
    pub(super) profile_id: String,
    pub(super) stages: Vec<SourceLeafArtifactEvidenceStage>,
}

#[derive(Debug, Serialize)]
pub(super) struct SourceLeafArtifactEvidenceStage {
    pub(super) leaf_id: String,
    pub(super) interaction: String,
    pub(super) stage_id: String,
    pub(super) media_path: String,
    pub(super) media_sha256: String,
    pub(super) pixel_hash: String,
    pub(super) root_record_hash: String,
    pub(super) accesskit_snapshot_hash: String,
    pub(super) unicode_evidence_hash: String,
    pub(super) root_revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ConsumerArtifactStageRequest {
    pub(super) leaf_id: String,
    pub(super) interaction: GenericInteractionClass,
}

impl ConsumerArtifactStageRequest {
    pub(crate) fn new(leaf_id: impl Into<String>, interaction: GenericInteractionClass) -> Self {
        Self {
            leaf_id: leaf_id.into(),
            interaction,
        }
    }
}

pub(super) fn validate_plan(
    plan: SourceLeafArtifactPlan,
) -> Result<ValidatedPlan, Box<dyn std::error::Error>> {
    validate_header(&plan)?;
    if plan.stages.is_empty() {
        return Err("source-leaf artifact plan must contain at least one stage".into());
    }
    let stages = stage_requests(plan.stages)?;
    Ok((plan.profile_id, stages))
}

fn validate_header(plan: &SourceLeafArtifactPlan) -> Result<(), Box<dyn std::error::Error>> {
    validate_schema(&plan.schema)?;
    validate_profile(&plan.profile_id)
}

fn validate_schema(schema: &str) -> Result<(), Box<dyn std::error::Error>> {
    if schema != PLAN_SCHEMA {
        return Err(format!("unsupported source-leaf artifact plan schema: {schema}").into());
    }
    Ok(())
}

fn validate_profile(profile_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    if profile_id.trim().is_empty() {
        return Err("source-leaf artifact plan profile_id must not be empty".into());
    }
    let expected_profile = current_profile_id()?;
    if profile_id != expected_profile {
        return Err(format!(
            "source-leaf artifact plan profile {profile_id} does not match this runner {expected_profile}"
        )
        .into());
    }
    Ok(())
}

fn stage_requests(
    input: Vec<SourceLeafArtifactStage>,
) -> Result<Vec<ConsumerArtifactStageRequest>, Box<dyn std::error::Error>> {
    let mut leaf_ids = BTreeSet::new();
    input
        .into_iter()
        .map(|stage| {
            if !leaf_ids.insert(stage.leaf_id.clone()) {
                return Err(format!(
                    "source-leaf artifact plan has duplicate leaf: {}",
                    stage.leaf_id
                )
                .into());
            }
            Ok(ConsumerArtifactStageRequest::new(
                stage.leaf_id,
                parse_interaction(&stage.interaction)?,
            ))
        })
        .collect()
}

pub(super) fn current_profile_id() -> Result<&'static str, Box<dyn std::error::Error>> {
    match std::env::consts::OS {
        "macos" => Ok("macos-latest"),
        "windows" => Ok("windows-latest"),
        "linux" => Ok("ubuntu-latest"),
        os => Err(format!("unsupported source-leaf artifact runner OS: {os}").into()),
    }
}

pub(super) fn interaction_name(interaction: GenericInteractionClass) -> &'static str {
    match interaction {
        GenericInteractionClass::TextInput => "text_input",
        GenericInteractionClass::ImeCommit => "ime_commit",
        GenericInteractionClass::Selection => "selection",
        GenericInteractionClass::Scroll => "scroll",
        GenericInteractionClass::ToolbarActivation => "toolbar_activation",
        GenericInteractionClass::FloatingToolbar => "floating_toolbar",
        GenericInteractionClass::Search => "search",
        GenericInteractionClass::ContextMenu => "context_menu",
        GenericInteractionClass::AccessibilityActivation => "accessibility_activation",
        GenericInteractionClass::ViewportResize => "viewport_resize",
    }
}

fn parse_interaction(value: &str) -> Result<GenericInteractionClass, Box<dyn std::error::Error>> {
    match value {
        "text_input" => Ok(GenericInteractionClass::TextInput),
        "ime_commit" => Ok(GenericInteractionClass::ImeCommit),
        "selection" => Ok(GenericInteractionClass::Selection),
        "scroll" => Ok(GenericInteractionClass::Scroll),
        "toolbar_activation" => Ok(GenericInteractionClass::ToolbarActivation),
        "floating_toolbar" => Ok(GenericInteractionClass::FloatingToolbar),
        "search" => Ok(GenericInteractionClass::Search),
        "context_menu" => Ok(GenericInteractionClass::ContextMenu),
        "accessibility_activation" => Ok(GenericInteractionClass::AccessibilityActivation),
        "viewport_resize" => Ok(GenericInteractionClass::ViewportResize),
        _ => Err(format!("unknown KUC generic interaction: {value}").into()),
    }
}
