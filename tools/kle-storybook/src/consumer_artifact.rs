use katana_ui_core::egui::text_command_surface::{
    ConsumerArtifactLeafId, ConsumerArtifactPlanIssuer, ConsumerArtifactPlanV1,
    ConsumerArtifactStageBinding, FullTextCommandSurfaceScenarioSession, GenericEffectClass,
    GenericInteractionClass, IssuedConsumerArtifactPlan,
};
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const KUC_ACTION_TARGET: &str = "kuc.rich.inline-strong";
#[path = "unicode_evidence.rs"]
mod unicode_evidence;

#[cfg(test)]
use unicode_evidence::REQUIRED_STAR_SCALARS;
use unicode_evidence::UnicodeEvidenceValidator;

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
        fs::create_dir_all(output_dir)?;
        let expected_stage_count = GenericInteractionClass::FULL_EDITOR_SEQUENCE.len();
        let mut plan = Self::issue_full_editor_plan()?;
        Self::execute_all_stages(&mut plan, output_dir, expected_stage_count)?;
        Ok(expected_stage_count)
    }

    fn issue_full_editor_plan() -> Result<IssuedConsumerArtifactPlan, Box<dyn std::error::Error>> {
        let session = FullTextCommandSurfaceScenarioSession::new_consumer_artifact();
        let mut leases = Vec::with_capacity(GenericInteractionClass::FULL_EDITOR_SEQUENCE.len());
        leases.push(session.retain_lease()?);
        for _ in 1..GenericInteractionClass::FULL_EDITOR_SEQUENCE.len() {
            leases.push(session.synchronize_lease()?);
        }
        let bindings = GenericInteractionClass::FULL_EDITOR_SEQUENCE
            .into_iter()
            .zip(leases)
            .enumerate()
            .map(|(index, (interaction, lease))| {
                Ok(ConsumerArtifactStageBinding::from_host_projection_lease(
                    ConsumerArtifactLeafId::new(format!("kle-full-editor-stage-{index}"))?,
                    KUC_ACTION_TARGET,
                    interaction,
                    GenericEffectClass::NoHostEffect,
                    lease,
                ))
            })
            .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
        ConsumerArtifactPlanIssuer::new()
            .issue(ConsumerArtifactPlanV1::new(1, bindings))
            .map_err(Into::into)
    }

    fn execute_all_stages(
        plan: &mut IssuedConsumerArtifactPlan,
        output_dir: &Path,
        expected_stage_count: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let context = egui::Context::default();
        for _ in 0..expected_stage_count {
            let evidence = plan.execute_next(&context, output_dir)?;
            UnicodeEvidenceValidator::validate(evidence.unicode_evidence_json())?;
        }
        if plan.remaining_stage_count() != 0 {
            return Err(format!(
                "KUC consumer artifact plan left {} stages",
                plan.remaining_stage_count()
            )
            .into());
        }
        Ok(())
    }

    fn run_nonce() -> Result<u128, Box<dyn std::error::Error>> {
        Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos())
    }

    fn run_output_dir(output_root: &Path, nonce: u128) -> PathBuf {
        output_root.join(format!("run-{}-{nonce}", std::process::id()))
    }
}

#[cfg(test)]
mod tests {
    use super::{ConsumerArtifactRunner, REQUIRED_STAR_SCALARS, UnicodeEvidenceValidator};
    use std::path::Path;

    #[test]
    fn exact_star_evidence_requires_scalar_and_variation_selector() {
        assert_eq!(REQUIRED_STAR_SCALARS, "[11088,65039]");
        assert_ne!(REQUIRED_STAR_SCALARS, "[11088]");
    }

    #[test]
    fn kuc_evidence_validator_requires_the_full_unicode_evidence_shape() {
        let evidence = br#"{
            "schema":"kuc.unicode-color-glyph-evidence",
            "schema_version":1,
            "graphemes":[{"scalar_sequence":[11088,65039]}],
            "ime":{"preedit_event_seen":true,"commit_event_seen":true,"preedit_scalar_sequence":[12363],"commit_scalar_sequence":[26085]},
            "caret":{"bounds":{"width":1,"height":26}},
            "hit_tests":[{"target":"star"}],
            "star":{"bounds":{"width":19,"height":26},"chromatic_pixel_count":1}
        }"#;
        assert!(UnicodeEvidenceValidator::validate(evidence).is_ok());
        assert!(UnicodeEvidenceValidator::validate(br#"{"graphemes":[]}"#).is_err());
    }

    #[test]
    fn kuc_evidence_validator_rejects_empty_unicode_evidence_sections() {
        let evidence = br#"{
            "schema":"kuc.unicode-color-glyph-evidence",
            "schema_version":1,
            "graphemes":[{"scalar_sequence":[11088,65039]}],
            "ime":{},"caret":{},"hit_tests":[],"star":{}
        }"#;
        assert!(UnicodeEvidenceValidator::validate(evidence).is_err());
    }

    #[test]
    fn consumer_artifacts_are_immutable_per_run() {
        let root = Path::new("target/acceptance/kle-storybook-consumer-artifact");
        assert_ne!(
            ConsumerArtifactRunner::run_output_dir(root, 1),
            ConsumerArtifactRunner::run_output_dir(root, 2)
        );
    }
}
