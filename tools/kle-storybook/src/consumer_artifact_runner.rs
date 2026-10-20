use crate::consumer_artifact::consumer_artifact_execution::execute_all_stages;
use crate::consumer_artifact::consumer_artifact_model::{
    ConsumerArtifactStageRequest, EVIDENCE_MANIFEST, EVIDENCE_SCHEMA, SourceLeafArtifactEvidence,
    SourceLeafArtifactEvidenceStage, SourceLeafArtifactPlan, validate_plan,
};
use katana_ui_core::egui::text_command_surface::{
    ConsumerArtifactLeafId, ConsumerArtifactPlanIssuer, ConsumerArtifactPlanV2,
    ConsumerArtifactStageBinding, FullTextCommandSurfaceScenarioSession, GenericEffectClass,
    GenericInteractionClass, IssuedConsumerArtifactPlan,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const KUC_ACTION_TARGET: &str = "kuc.rich.inline-strong";
const CONTRACT_STAGE_PREFIX: &str = "kle-contract-stage";

pub(crate) struct ConsumerArtifactRunner;

impl ConsumerArtifactRunner {
    pub(crate) fn write_full_editor_artifact_for_run(
        output_root: &Path,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let output_dir = Self::run_output_dir(output_root, Self::run_nonce()?);
        Self::write_full_editor_artifact(&output_dir)?;
        Ok(output_dir)
    }

    pub(crate) fn write_full_editor_artifact(
        output_dir: &Path,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let stages = Self::contract_stage_requests();
        Self::write_artifact_for_stages(output_dir, &stages)
    }

    pub(crate) fn write_source_leaf_artifact(
        input_path: &Path,
        output_dir: &Path,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let plan: SourceLeafArtifactPlan = serde_json::from_slice(&fs::read(input_path)?)?;
        let validated_plan = validate_plan(plan)?;
        if output_dir.exists() {
            return Err(format!(
                "refusing to overwrite source-leaf consumer artifact output: {}",
                output_dir.display()
            )
            .into());
        }
        let evidence = Self::write_artifact_with_evidence(output_dir, &validated_plan.stages)?;
        let manifest = SourceLeafArtifactEvidence {
            schema: EVIDENCE_SCHEMA,
            provenance: validated_plan.provenance,
            profile_id: validated_plan.profile_id,
            stages: evidence,
        };
        fs::write(
            output_dir.join(EVIDENCE_MANIFEST),
            serde_json::to_vec_pretty(&manifest)?,
        )?;
        Ok(output_dir.to_path_buf())
    }

    pub(crate) fn write_artifact_for_stages(
        output_dir: &Path,
        stages: &[ConsumerArtifactStageRequest],
    ) -> Result<usize, Box<dyn std::error::Error>> {
        Self::write_artifact_with_evidence(output_dir, stages)?;
        Ok(stages.len())
    }

    fn write_artifact_with_evidence(
        output_dir: &Path,
        stages: &[ConsumerArtifactStageRequest],
    ) -> Result<Vec<SourceLeafArtifactEvidenceStage>, Box<dyn std::error::Error>> {
        fs::create_dir_all(output_dir)?;
        let mut plan = Self::issue_plan(stages)?;
        execute_all_stages(&mut plan, output_dir, stages)
    }

    pub(super) fn contract_stage_requests() -> Vec<ConsumerArtifactStageRequest> {
        GenericInteractionClass::FULL_EDITOR_SEQUENCE
            .into_iter()
            .chain(std::iter::once(GenericInteractionClass::ImeCommit))
            .enumerate()
            .map(|(index, interaction)| {
                ConsumerArtifactStageRequest::new(
                    format!("{CONTRACT_STAGE_PREFIX}-{index}"),
                    interaction,
                )
            })
            .collect()
    }

    fn issue_plan(
        stages: &[ConsumerArtifactStageRequest],
    ) -> Result<IssuedConsumerArtifactPlan, Box<dyn std::error::Error>> {
        let session = FullTextCommandSurfaceScenarioSession::new_consumer_artifact();
        let mut leases = Vec::with_capacity(stages.len());
        leases.push(session.retain_lease()?);
        for _ in 1..stages.len() {
            leases.push(session.synchronize_lease()?);
        }
        let bindings = stages
            .iter()
            .zip(leases)
            .map(|(stage, lease)| {
                Ok(ConsumerArtifactStageBinding::from_host_projection_lease(
                    ConsumerArtifactLeafId::new(stage.leaf_id.clone())?,
                    KUC_ACTION_TARGET,
                    stage.interaction,
                    GenericEffectClass::NoHostEffect,
                    lease,
                ))
            })
            .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
        ConsumerArtifactPlanIssuer::new()
            .issue_v2(ConsumerArtifactPlanV2::new(1, bindings))
            .map_err(Into::into)
    }

    pub(super) fn run_output_dir(output_root: &Path, nonce: u128) -> PathBuf {
        output_root.join(format!("run-{}-{nonce}", std::process::id()))
    }

    fn run_nonce() -> Result<u128, Box<dyn std::error::Error>> {
        Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos())
    }
}
