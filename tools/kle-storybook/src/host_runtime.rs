use crate::args::StorybookMode;
use crate::host_types::{STORYBOOK_VIEWPORT_HEIGHT, STORYBOOK_VIEWPORT_WIDTH, StorybookHost};
use crate::search_motion_sequence::SearchMotionSequence;
use std::path::Path;

impl StorybookHost {
    pub(crate) fn render_live_frame(
        &mut self,
        output_dir: &Path,
        stage_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut frame_error = None;
        let context = egui::Context::default();
        let mut output = context.run_ui(live_raw_input(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                frame_error = render_frame_error(self, ui, output_dir, stage_id);
            });
        });
        /* WHY: Headless evidence is emitted by the KUC root; no egui texture backend exists here. */
        output.textures_delta.clear();
        frame_error.map_or(Ok(()), |error| Err(error.into()))
    }

    pub(crate) fn run_live_check(
        &mut self,
        _mode: StorybookMode,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.render_live_frame(Path::new("target/kle-storybook-live"), "check-0000")
    }

    pub(crate) fn write_live_artifact(
        &mut self,
        output_dir: &Path,
        frames: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let frame_count = frames.max(1);
        for frame in 0..frame_count {
            self.render_live_frame(output_dir, &format!("frame-{frame:04}"))?;
        }
        Ok(())
    }

    pub(crate) fn write_search_motion_sequence(
        &mut self,
        output: &Path,
        frames: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        SearchMotionSequence::write(self, output, frames)
    }
}

fn live_raw_input() -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(
                STORYBOOK_VIEWPORT_WIDTH as f32,
                STORYBOOK_VIEWPORT_HEIGHT as f32,
            ),
        )),
        ..egui::RawInput::default()
    }
}

fn render_frame_error(
    host: &mut StorybookHost,
    ui: &mut egui::Ui,
    output_dir: &Path,
    stage_id: &str,
) -> Option<String> {
    let mut no_op =
        |_locator: &katana_ui_core::egui::text_command_surface::KucInteractionLocator| {};
    match host.show(ui, output_dir, stage_id, &mut no_op) {
        Ok(frame) => frame.validate_live_evidence().err(),
        Err(error) => Some(error.to_string()),
    }
}
