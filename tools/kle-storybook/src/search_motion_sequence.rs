use crate::host_types::StorybookHost;
#[path = "search_motion_sequence_frame.rs"]
mod frame;
use egui::RawInput;
use frame::{MotionArtifactFrame, capture_stage_id, prepare_output};
use katana_ui_core::egui::text_command_surface::FullTextCommandSurfaceMotionPlan;
use katana_ui_core::egui::{MotionArtifactWriter, OpaqueMotionReceiptSequence};
use std::path::Path;

#[path = "search_motion_manifest.rs"]
mod manifest;

pub(crate) struct SearchMotionSequence;

impl SearchMotionSequence {
    pub(crate) fn write(
        host: &mut StorybookHost,
        output_dir: &Path,
        requested_frames: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let plan = FullTextCommandSurfaceMotionPlan::issue(requested_frames)?;
        prepare_output(output_dir)?;
        manifest::invalidate_completion(output_dir)?;
        let frames = Self::render_frames(host, output_dir, &plan)?;
        Self::validate_scenario(&frames, &plan)?;
        let artifact = MotionArtifactWriter::new()
            .write_opaque_variable_viewport(&opaque_sequence(&frames)?, output_dir)?;
        manifest::write(output_dir, requested_frames, &frames, &artifact)
    }

    fn render_frames(
        host: &mut StorybookHost,
        receipt_dir: &Path,
        plan: &FullTextCommandSurfaceMotionPlan,
    ) -> Result<Vec<MotionArtifactFrame>, Box<dyn std::error::Error>> {
        let context = egui::Context::default();
        let mut frames = Vec::with_capacity(plan.frames().len());
        let mut active_scenario = None;
        let mut continuation = None;
        for motion_frame in plan.frames() {
            Self::activate_scenario(host, &mut active_scenario, motion_frame.scenario_id())?;
            frames.push(Self::render_motion_frame(
                &context,
                host,
                receipt_dir,
                motion_frame,
                frames.len(),
                &mut continuation,
            )?);
        }
        Ok(frames)
    }

    fn activate_scenario(
        host: &mut StorybookHost,
        active_scenario: &mut Option<
            katana_ui_core::egui::text_command_surface::FullTextCommandSurfaceScenarioId,
        >,
        scenario_id: katana_ui_core::egui::text_command_surface::FullTextCommandSurfaceScenarioId,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if *active_scenario != Some(scenario_id) {
            *host = StorybookHost::for_scenario(scenario_id)?;
            *active_scenario = Some(scenario_id);
        }
        Ok(())
    }

    fn render_motion_frame(
        context: &egui::Context,
        host: &mut StorybookHost,
        receipt_dir: &Path,
        motion_frame: &katana_ui_core::egui::text_command_surface::FullTextCommandSurfaceMotionFrame,
        frame_index: usize,
        continuation: &mut Option<
            katana_ui_core::egui::text_command_surface::KucOpaqueMotionContinuation,
        >,
    ) -> Result<MotionArtifactFrame, Box<dyn std::error::Error>> {
        let mut input = RawInput::default();
        motion_frame.apply_to(&mut input, continuation)?;
        let stage_id = capture_stage_id(frame_index);
        let mut continuation_error = None;
        let frame = Self::render(
            context,
            host,
            receipt_dir,
            &stage_id,
            motion_frame.provenance_id().to_string(),
            input,
            &mut |locator| {
                continuation_error = motion_frame
                    .capture_continuation(locator, continuation)
                    .err()
            },
        )?;
        continuation_error.map_or(Ok(frame), |error| Err(error.into()))
    }

    fn render(
        context: &egui::Context,
        host: &mut StorybookHost,
        output_dir: &Path,
        stage_id: &str,
        scenario_stage: String,
        input: RawInput,
        callback: &mut dyn FnMut(
            &katana_ui_core::egui::text_command_surface::KucInteractionLocator,
        ),
    ) -> Result<MotionArtifactFrame, Box<dyn std::error::Error>> {
        let mut result = None;
        let mut output = context.run_ui(input, |ui| {
            result = Some(host.show(ui, output_dir, stage_id, callback));
        });
        /* WHY: Motion artifacts use KUC receipts instead of an egui texture backend. */
        output.textures_delta.clear();
        let root = result.ok_or_else(|| "motion artifact root did not render".to_string())??;
        root.validate_live_evidence()
            .map_err(std::io::Error::other)?;
        MotionArtifactFrame::from_root(root, stage_id, scenario_stage)
    }

    fn validate_scenario(
        frames: &[MotionArtifactFrame],
        plan: &FullTextCommandSurfaceMotionPlan,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if frames.len() != plan.frames().len() {
            return Err("encoded motion frame count diverged from the KUC motion plan".into());
        }
        if frames.iter().any(|frame| {
            frame.receipt_record_hash.is_empty()
                || frame.scenario_stage.is_empty()
                || frame.scenario_stage == "idle"
        }) {
            return Err(
                "motion artifact contains incomplete KUC provenance or receipt evidence".into(),
            );
        }
        Ok(())
    }
}

fn opaque_sequence(
    frames: &[MotionArtifactFrame],
) -> Result<OpaqueMotionReceiptSequence, Box<dyn std::error::Error>> {
    let mut sequence = OpaqueMotionReceiptSequence::new();
    for frame in frames {
        sequence.push(&frame.stage_id, frame.artifact_receipt.clone())?;
    }
    Ok(sequence)
}
