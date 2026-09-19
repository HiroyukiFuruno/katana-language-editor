#[path = "root_frame_evidence.rs"]
mod root_frame_evidence;

pub(crate) use crate::host_types_runtime::StorybookHost;
pub(crate) use root_frame_evidence::StorybookRootFrame;

use katana_language_editor_egui::HostProjectionProvider;
use katana_language_editor_egui::HostProjectionProviderError;
use katana_ui_core::egui::text_command_surface::{
    EguiTextCommandSurfaceHostProjectionLease, FullTextCommandSurfaceRawInputStage,
    FullTextCommandSurfaceScenarioFactory, FullTextCommandSurfaceScenarioId,
    FullTextCommandSurfaceScenarioSession,
};

pub(crate) const STORYBOOK_VIEWPORT_WIDTH: u32 = 1280;
pub(crate) const STORYBOOK_VIEWPORT_HEIGHT: u32 = 720;

pub(crate) struct StorybookProjectionProvider {
    session: FullTextCommandSurfaceScenarioSession,
}

impl Default for StorybookProjectionProvider {
    fn default() -> Self {
        Self {
            session: FullTextCommandSurfaceScenarioSession::new(
                FullTextCommandSurfaceScenarioId::WorkspaceTabs,
            ),
        }
    }
}

impl StorybookProjectionProvider {
    pub(crate) fn for_scenario(
        scenario_id: FullTextCommandSurfaceScenarioId,
    ) -> Result<(Self, Vec<FullTextCommandSurfaceRawInputStage>), String> {
        let scenario = FullTextCommandSurfaceScenarioFactory::new()
            .issue(scenario_id)
            .map_err(|error| error.to_string())?;
        let stages = scenario.stages().to_vec();
        Ok((
            Self {
                session: FullTextCommandSurfaceScenarioSession::new(scenario_id),
            },
            stages,
        ))
    }

    fn retain_projection_lease(&self) -> Result<EguiTextCommandSurfaceHostProjectionLease, String> {
        self.session
            .retain_lease()
            .map_err(|error| error.to_string())
    }

    fn synchronize_projection_lease(
        &self,
    ) -> Result<EguiTextCommandSurfaceHostProjectionLease, String> {
        self.session
            .synchronize_lease()
            .map_err(|error| error.to_string())
    }
}

impl HostProjectionProvider for StorybookProjectionProvider {
    type Lease = EguiTextCommandSurfaceHostProjectionLease;
    type Error = String;

    fn retain_lease(
        &mut self,
    ) -> Result<EguiTextCommandSurfaceHostProjectionLease, HostProjectionProviderError<Self::Error>>
    {
        self.retain_projection_lease()
            .map_err(HostProjectionProviderError::Provider)
    }

    fn synchronize_lease(
        &mut self,
    ) -> Result<EguiTextCommandSurfaceHostProjectionLease, HostProjectionProviderError<Self::Error>>
    {
        self.synchronize_projection_lease()
            .map_err(HostProjectionProviderError::Provider)
    }
}
