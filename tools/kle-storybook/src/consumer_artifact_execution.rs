use crate::consumer_artifact::consumer_artifact_model::{
    ConsumerArtifactStageRequest, SourceLeafArtifactEvidenceStage, interaction_name,
};
use crate::consumer_artifact::unicode_evidence::UnicodeEvidenceValidator;
use katana_ui_core::egui::text_command_surface::IssuedConsumerArtifactPlan;
use std::path::Path;

pub(super) fn execute_all_stages(
    plan: &mut IssuedConsumerArtifactPlan,
    output_dir: &Path,
    stages: &[ConsumerArtifactStageRequest],
) -> Result<Vec<SourceLeafArtifactEvidenceStage>, Box<dyn std::error::Error>> {
    let context = egui::Context::default();
    let mut records = Vec::with_capacity(stages.len());
    for stage in stages {
        let evidence = plan.execute_next(&context, output_dir)?;
        UnicodeEvidenceValidator::validate(evidence.unicode_evidence_json())?;
        let record = record_stage(&evidence, output_dir, stage)?;
        plan.consume_forwarding_receipt_once(evidence.into_forwarding_receipt())?;
        records.push(record);
    }
    if plan.remaining_stage_count() != 0 {
        return Err(format!(
            "KUC consumer artifact plan left {} stages",
            plan.remaining_stage_count()
        )
        .into());
    }
    Ok(records)
}

fn record_stage(
    evidence: &katana_ui_core::egui::text_command_surface::ConsumerArtifactEvidence,
    output_dir: &Path,
    stage: &ConsumerArtifactStageRequest,
) -> Result<SourceLeafArtifactEvidenceStage, Box<dyn std::error::Error>> {
    if evidence.leaf().as_str() != stage.leaf_id {
        return Err("KUC evidence leaf does not match the issued source leaf".into());
    }
    let stage_id = evidence.stage_id().to_owned();
    let media_path = format!("{stage_id}.png");
    if !output_dir.join(&media_path).is_file() {
        return Err(format!("KUC stage media is missing: {media_path}").into());
    }
    Ok(SourceLeafArtifactEvidenceStage {
        leaf_id: stage.leaf_id.clone(),
        interaction: interaction_name(stage.interaction).to_string(),
        stage_id,
        media_path,
        media_sha256: evidence.png_sha256().to_owned(),
        pixel_hash: evidence.pixel_hash().to_owned(),
        root_record_hash: evidence.root_record_hash().to_owned(),
        accesskit_snapshot_hash: evidence.accesskit_snapshot_hash().to_owned(),
        unicode_evidence_hash: evidence.unicode_evidence_hash().to_owned(),
        root_revision: evidence.root_revision(),
    })
}
