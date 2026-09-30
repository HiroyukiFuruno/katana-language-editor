use eframe36::egui;
use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};
use katana_platform::{DefaultCacheService, JsonFileRepository, SettingsService};
use katana_ui::{app_state::AppState, shell::KatanaApp};
use std::path::{Path, PathBuf};

use crate::host_target_locator::CurrentHostTarget;
use crate::{
    AccessKitMode, FrameCaptureError, FrameEvidence, HostTargetLocator, PhysicalFrameCapture,
};

#[derive(Debug, Eq, PartialEq)]
pub enum InitialFrameError {
    SandboxUnavailable,
    FrameCapture(FrameCaptureError),
}

impl std::fmt::Display for InitialFrameError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SandboxUnavailable => formatter.write_str("fixed host sandbox is unavailable"),
            Self::FrameCapture(error) => write!(formatter, "initial host frame failed: {error}"),
        }
    }
}

impl std::error::Error for InitialFrameError {}

impl From<FrameCaptureError> for InitialFrameError {
    fn from(error: FrameCaptureError) -> Self {
        Self::FrameCapture(error)
    }
}

pub struct FixedHostSession {
    app: KatanaApp,
    context: egui::Context,
    sandbox: PathBuf,
    current_target: Option<CurrentHostTarget>,
}

impl FixedHostSession {
    pub fn new(sandbox: impl AsRef<Path>) -> Self {
        let sandbox = sandbox.as_ref().to_path_buf();
        let ai_registry = AiProviderRegistry::new();
        let plugin_registry = PluginRegistry::new();
        let repo = JsonFileRepository::new(sandbox.join("settings.json"));
        let mut settings = SettingsService::new(Box::new(repo));
        settings.apply_os_default_theme();
        settings.apply_os_default_language();
        let cache = std::sync::Arc::new(DefaultCacheService::new(sandbox.join("cache.json")));
        let state = AppState::new(ai_registry, plugin_registry, settings, cache);

        let context = egui::Context::default();
        context.enable_accesskit();
        Self {
            app: KatanaApp::new(state),
            context,
            sandbox,
            current_target: None,
        }
    }

    pub fn initial_frame(&mut self) -> Result<FrameEvidence, InitialFrameError> {
        if !self.sandbox.is_dir() {
            return Err(InitialFrameError::SandboxUnavailable);
        }
        let capture = PhysicalFrameCapture::new(AccessKitMode::Required);
        self.current_target = None;
        capture
            .capture_katana_frame(&mut self.app, &self.context, egui::RawInput::default())
            .map_err(InitialFrameError::from)
    }

    pub fn current_frame_for_target(
        &mut self,
        locator: &HostTargetLocator,
    ) -> Result<FrameEvidence, InitialFrameError> {
        if !self.sandbox.is_dir() {
            return Err(InitialFrameError::SandboxUnavailable);
        }
        self.current_target = None;
        let capture = PhysicalFrameCapture::new(AccessKitMode::Required);
        let (evidence, target) = capture.capture_katana_frame_with_target(
            &mut self.app,
            &self.context,
            egui::RawInput::default(),
            Some(locator),
        )?;
        if let Some(target) = &target {
            let _ = (target.frame_hash, target.node_id, target.bounds);
        }
        self.current_target = target;
        Ok(evidence)
    }
}

#[cfg(test)]
mod tests {
    use super::FixedHostSession;
    use std::path::Path;

    #[test]
    fn session_api_does_not_expose_host_state() {
        fn construct(path: &Path) -> FixedHostSession {
            FixedHostSession::new(path)
        }

        let _constructor: fn(&Path) -> FixedHostSession = construct;
        let _ = FixedHostSession::initial_frame;
    }
}
