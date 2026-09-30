#[cfg(test)]
use crate::host_types::{STORYBOOK_VIEWPORT_HEIGHT, STORYBOOK_VIEWPORT_WIDTH};
use crate::host_types::{StorybookProjectionProvider, StorybookRootFrame};
use crate::root_runtime::{StorybookProjection, StorybookRootError};
#[cfg(test)]
use katana_ui_core::egui::text_command_surface::FullTextCommandSurfaceRawInputStage;
use katana_ui_core::egui::text_command_surface::FullTextCommandSurfaceScenarioId;
use std::path::Path;

pub(crate) struct StorybookHost {
    pub(crate) projection: Box<dyn StorybookProjection>,
    #[cfg(test)]
    stages: Vec<FullTextCommandSurfaceRawInputStage>,
}

impl StorybookHost {
    pub(crate) fn new() -> Result<Self, StorybookRootError> {
        Self::for_scenario(FullTextCommandSurfaceScenarioId::WorkspaceTabs)
    }

    pub(crate) fn for_scenario(
        scenario_id: FullTextCommandSurfaceScenarioId,
    ) -> Result<Self, StorybookRootError> {
        let (provider, stages) = StorybookProjectionProvider::for_scenario(scenario_id)
            .map_err(StorybookRootError::Binding)?;
        let projection = crate::root_runtime::InjectedProjection::new(provider)?;
        #[cfg(not(test))]
        let _ = stages;
        Ok(Self {
            projection: Box::new(projection),
            #[cfg(test)]
            stages,
        })
    }

    #[cfg(test)]
    pub(crate) fn scenario_stages(&self) -> &[FullTextCommandSurfaceRawInputStage] {
        &self.stages
    }

    pub(crate) fn show(
        &mut self,
        ui: &mut egui::Ui,
        output_dir: &Path,
        stage_id: &str,
        callback: &mut dyn FnMut(
            &katana_ui_core::egui::text_command_surface::KucInteractionLocator,
        ),
    ) -> Result<StorybookRootFrame, StorybookRootError> {
        self.projection
            .show_write_artifact_and_forward_once(ui, output_dir, stage_id, callback)
    }

    pub(crate) fn show_interactive(
        &mut self,
        ui: &mut egui::Ui,
    ) -> Result<katana_language_editor_egui::KucRootBindingReceipt, StorybookRootError> {
        self.projection.show_interactive(ui)
    }

    #[cfg(test)]
    pub(crate) fn render_scenario_frames(
        &mut self,
        output_dir: &Path,
        scenario_id: FullTextCommandSurfaceScenarioId,
    ) -> Result<Vec<StorybookRootFrame>, Box<dyn std::error::Error>> {
        let context = egui::Context::default();
        let stages = self.stages.clone();
        stages
            .iter()
            .enumerate()
            .map(|(index, stage)| {
                self.render_scenario_frame(&context, output_dir, scenario_id, index, stage)
            })
            .collect()
    }

    #[cfg(test)]
    fn render_scenario_frame(
        &mut self,
        context: &egui::Context,
        output_dir: &Path,
        scenario_id: FullTextCommandSurfaceScenarioId,
        index: usize,
        stage: &FullTextCommandSurfaceRawInputStage,
    ) -> Result<StorybookRootFrame, Box<dyn std::error::Error>> {
        let mut input = scenario_input();
        stage.apply_to(&mut input);
        let mut result = None;
        let mut no_op =
            |_locator: &katana_ui_core::egui::text_command_surface::KucInteractionLocator| {};
        let mut output = context.run_ui(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                result = Some(self.show(
                    ui,
                    output_dir,
                    &format!("{scenario_id:?}-{index:04}"),
                    &mut no_op,
                ));
            });
        });
        /* WHY: Headless evidence is emitted by the KUC root; no egui texture backend exists here. */
        output.textures_delta.clear();
        Ok(result.ok_or_else(|| "scenario root did not render".to_string())??)
    }
}

#[cfg(test)]
fn scenario_input() -> egui::RawInput {
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
