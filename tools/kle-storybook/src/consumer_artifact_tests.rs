use super::ConsumerArtifactRunner;
use super::consumer_artifact_model::{
    ConsumerArtifactStageRequest, EVIDENCE_MANIFEST, PLAN_SCHEMA, SourceLeafArtifactPlan,
    SourceLeafArtifactStage, current_profile_id, validate_plan,
};
use super::consumer_artifact_provenance_tests::provenance;
use super::unicode_evidence::{REQUIRED_STAR_SCALARS, UnicodeEvidenceValidator};
use katana_ui_core::egui::text_command_surface::GenericInteractionClass;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn exact_star_evidence_requires_scalar_and_variation_selector() {
    assert_eq!(REQUIRED_STAR_SCALARS, "[11088,65039]");
    assert_ne!(REQUIRED_STAR_SCALARS, "[11088]");
}

#[test]
fn kuc_evidence_validator_requires_complete_shape() {
    let evidence = br#"{"schema":"kuc.unicode-color-glyph-evidence","schema_version":1,"graphemes":[{"scalar_sequence":[11088,65039]}],"ime":{"preedit_event_seen":true,"commit_event_seen":true,"preedit_scalar_sequence":[12363],"commit_scalar_sequence":[26085]},"caret":{"bounds":{"width":1,"height":26}},"hit_tests":[{"target":"star"}],"star":{"bounds":{"width":19,"height":26},"chromatic_pixel_count":1}}"#;
    assert!(UnicodeEvidenceValidator::validate(evidence).is_ok());
    assert!(UnicodeEvidenceValidator::validate(br#"{"graphemes":[]}"#).is_err());
}

#[test]
fn consumer_artifacts_are_immutable_per_run() {
    let root = Path::new("target/acceptance/kle-storybook-consumer-artifact");
    assert_ne!(
        ConsumerArtifactRunner::run_output_dir(root, 1),
        ConsumerArtifactRunner::run_output_dir(root, 2)
    );
}

#[test]
fn source_leaf_plan_rejects_invalid_bindings() -> Result<(), Box<dyn std::error::Error>> {
    for plan in [duplicate_plan()?, unknown_plan()?, profile_drift_plan()] {
        assert!(validate_plan(plan).is_err());
    }
    Ok(())
}

fn duplicate_plan() -> Result<SourceLeafArtifactPlan, Box<dyn std::error::Error>> {
    source_plan(vec![
        stage("leaf-a", "text_input"),
        stage("leaf-a", "search"),
    ])
}

fn unknown_plan() -> Result<SourceLeafArtifactPlan, Box<dyn std::error::Error>> {
    source_plan(vec![stage("leaf-a", "katana_action")])
}

fn profile_drift_plan() -> SourceLeafArtifactPlan {
    SourceLeafArtifactPlan {
        schema: PLAN_SCHEMA.to_string(),
        provenance: provenance(),
        profile_id: "unsupported-profile".to_string(),
        stages: vec![stage("leaf-a", "text_input")],
    }
}

fn source_plan(
    stages: Vec<SourceLeafArtifactStage>,
) -> Result<SourceLeafArtifactPlan, Box<dyn std::error::Error>> {
    Ok(SourceLeafArtifactPlan {
        schema: PLAN_SCHEMA.to_string(),
        provenance: provenance(),
        profile_id: current_profile_id()?.to_string(),
        stages,
    })
}

fn stage(leaf_id: &str, interaction: &str) -> SourceLeafArtifactStage {
    SourceLeafArtifactStage {
        leaf_id: leaf_id.to_string(),
        interaction: interaction.to_string(),
    }
}

#[test]
fn source_leaf_producer_writes_kuc_owned_media_and_evidence()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temporary_root()?;
    let output = write_producer_artifact(&root)?;
    assert_producer_artifact(&output)?;
    fs::remove_dir_all(root)?;
    Ok(())
}

fn temporary_root() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    Ok(std::env::temp_dir().join(format!("kle-source-leaf-producer-{nonce}")))
}

fn write_producer_artifact(root: &Path) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let input = root.join("plan.json");
    let output = root.join("artifact");
    fs::create_dir_all(root)?;
    fs::write(&input, serde_json::to_vec(&producer_plan()?)?)?;
    ConsumerArtifactRunner::write_source_leaf_artifact(&input, &output)?;
    Ok(output)
}

fn producer_plan() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    Ok(
        serde_json::json!({"schema": PLAN_SCHEMA, "run_id": "producer-test-run", "source_closure_sha256": format!("sha256:{}", "c".repeat(64)), "profile_fingerprint": format!("sha256:{}", "d".repeat(64)), "profile_id": current_profile_id()?, "stages": [
            {"leaf_id": "source-leaf-input", "interaction": "text_input"},
            {"leaf_id": "source-leaf-ime", "interaction": "ime_commit"}
        ]}),
    )
}

fn assert_producer_artifact(output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let evidence: serde_json::Value =
        serde_json::from_slice(&fs::read(output.join(EVIDENCE_MANIFEST))?)?;
    let stages = evidence["stages"]
        .as_array()
        .ok_or("missing stage evidence")?;
    assert_eq!(evidence["run_id"], "producer-test-run");
    assert_eq!(
        evidence["source_closure_sha256"],
        format!("sha256:{}", "c".repeat(64))
    );
    assert_eq!(
        evidence["profile_fingerprint"],
        format!("sha256:{}", "d".repeat(64))
    );
    assert_eq!(stages.len(), 2);
    for (index, leaf) in ["source-leaf-input", "source-leaf-ime"].iter().enumerate() {
        assert_stage(&stages[index], output, leaf)?;
    }
    Ok(())
}

fn assert_stage(
    evidence: &serde_json::Value,
    output: &Path,
    leaf: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(evidence["leaf_id"], leaf);
    let media = evidence["media_path"]
        .as_str()
        .ok_or("missing media path")?;
    assert!(output.join(media).is_file());
    assert_hash(evidence, "media_sha256");
    assert_hash(evidence, "accesskit_snapshot_hash");
    Ok(())
}

fn assert_hash(evidence: &serde_json::Value, field: &str) {
    assert!(
        evidence[field]
            .as_str()
            .is_some_and(|value| value.len() == 64)
    );
}

#[test]
fn contract_plan_is_variable_length_and_distinct() {
    let stages = ConsumerArtifactRunner::contract_stage_requests();
    assert!(stages.len() > GenericInteractionClass::FULL_EDITOR_SEQUENCE.len());
    let ids = stages
        .iter()
        .map(|stage| stage.leaf_id.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), stages.len());
}

#[test]
fn stage_request_preserves_opaque_leaf_identity() {
    let stage = ConsumerArtifactStageRequest::new(
        "source-leaf:opaque-fingerprint",
        GenericInteractionClass::Search,
    );
    assert_eq!(stage.leaf_id, "source-leaf:opaque-fingerprint");
    assert_eq!(stage.interaction, GenericInteractionClass::Search);
}
