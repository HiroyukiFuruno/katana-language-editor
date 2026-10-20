use katana_language_editor::EditorResult;
use katana_language_editor_egui::{
    EguiTextCommandSurfaceEditor, HostProjectionProvider, HostProjectionProviderError,
};
use katana_ui_core::atom::TextArea;
use katana_ui_core::egui::text_command_surface::{
    EguiTextCommandSurfaceHostProjectionEncoder, EguiTextCommandSurfaceHostProjectionLease,
    EguiTextCommandSurfacePresentation, TextCommandSurfaceStyle,
};
use katana_ui_core::render_model::UiTextSpan;
use katana_ui_core::text_surface::{
    TextSurface, TextSurfacePresentation, TextSurfaceProps, TextSurfaceViewport,
};

const ROOT_IDENTITY: &str = "katana-downstream-check.neutral";
const NEUTRAL_VIEWPORT_WIDTH: u32 = 800;
const NEUTRAL_VIEWPORT_HEIGHT: u32 = 600;

/// Diagnostic result for the public KLE root boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PublicEditorDiagnostic {
    public_editor_constructed: bool,
    fixed_katana_host_evidence: bool,
    semantic_action_surface_constructed: bool,
}

impl PublicEditorDiagnostic {
    pub(crate) fn construct() -> EditorResult<Self> {
        let _editor =
            EguiTextCommandSurfaceEditor::new(NeutralProjectionProvider).map_err(|error| {
                katana_language_editor::EditorError::Internal(format!(
                    "public KLE editor construction failed: {error:?}"
                ))
            })?;
        Ok(Self {
            public_editor_constructed: true,
            fixed_katana_host_evidence: false,
            semantic_action_surface_constructed: false,
        })
    }

    pub(crate) fn summary(self) -> String {
        format!(
            "katana-downstream-check: public_kle_editor_constructed={} fixed_katana_host_evidence={} semantic_action_surface_constructed={} (diagnostic-only; not fixed-KatanA host evidence)",
            self.public_editor_constructed,
            self.fixed_katana_host_evidence,
            self.semantic_action_surface_constructed,
        )
    }
}

#[derive(Default)]
struct NeutralProjectionProvider;

impl NeutralProjectionProvider {
    fn lease() -> Result<EguiTextCommandSurfaceHostProjectionLease, String> {
        let text_area = TextArea::new(ROOT_IDENTITY).value(String::new());
        let props = TextSurfaceProps::new(
            text_area,
            Vec::<UiTextSpan>::new(),
            TextSurfaceViewport::new(0, 0, NEUTRAL_VIEWPORT_WIDTH, NEUTRAL_VIEWPORT_HEIGHT),
        );
        let surface = TextSurface::new(props);
        let mut text = TextSurfacePresentation::from_props(surface.props());
        text.accessibility_label = "Neutral KLE diagnostic surface".to_string();
        let presentation = EguiTextCommandSurfacePresentation {
            text_state_id: Some(ROOT_IDENTITY.into()),
            text,
            toolbar: None,
            floating: None,
            search: None,
            context_menu: None,
        };
        let style = TextCommandSurfaceStyle::standard()
            .map_err(|error| format!("failed to construct neutral text surface style: {error}"))?;
        let token = EguiTextCommandSurfaceHostProjectionEncoder::token(
            1,
            ROOT_IDENTITY.as_bytes().to_vec(),
            presentation,
            style,
        )
        .map_err(|error| error.to_string())?;
        Ok(EguiTextCommandSurfaceHostProjectionLease::new(
            token,
            |_context| Ok(None),
        ))
    }
}

impl HostProjectionProvider for NeutralProjectionProvider {
    type Lease = EguiTextCommandSurfaceHostProjectionLease;
    type Error = String;

    fn retain_lease(
        &mut self,
    ) -> Result<EguiTextCommandSurfaceHostProjectionLease, HostProjectionProviderError<Self::Error>>
    {
        Self::lease().map_err(HostProjectionProviderError::Provider)
    }

    fn synchronize_lease(
        &mut self,
    ) -> Result<EguiTextCommandSurfaceHostProjectionLease, HostProjectionProviderError<Self::Error>>
    {
        Self::lease().map_err(HostProjectionProviderError::Provider)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_constructs_public_editor_without_semantic_surface() -> EditorResult<()> {
        let diagnostic = PublicEditorDiagnostic::construct()?;
        assert!(diagnostic.public_editor_constructed);
        assert!(!diagnostic.fixed_katana_host_evidence);
        assert!(!diagnostic.semantic_action_surface_constructed);
        assert!(diagnostic.summary().contains("diagnostic-only"));
        assert!(
            diagnostic
                .summary()
                .contains("not fixed-KatanA host evidence")
        );
        Ok(())
    }
}
