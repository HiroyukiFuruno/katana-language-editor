use crate::KucRootBindingReceipt;
use crate::host_projection_provider::{
    HostProjectionBinding, HostProjectionBindingError, HostProjectionProvider,
};
use egui::Ui;
use katana_ui_core::egui::text_command_surface::EguiTextCommandSurfaceHostProjectionLease;
use katana_ui_core::egui::text_command_surface::KucInteractionLocator;
use katana_ui_core::egui::text_command_surface::KucRootEventBatchForwarder;

/// Public egui entry point for the opaque KUC root frame runtime.
///
/// ```compile_fail
/// use katana_language_editor::{HostProjectionProvider, HostProjectionProviderError};
/// use katana_language_editor_egui::EguiTextCommandSurfaceEditor;
///
/// struct WrongLease;
/// struct Provider;
///
/// impl HostProjectionProvider for Provider {
///     type Lease = WrongLease;
///     type Error = ();
///
///     fn retain_lease(&mut self) -> Result<Self::Lease, HostProjectionProviderError<Self::Error>> {
///         Ok(WrongLease)
///     }
///
///     fn synchronize_lease(
///         &mut self,
///     ) -> Result<Self::Lease, HostProjectionProviderError<Self::Error>> {
///         Ok(WrongLease)
///     }
/// }
///
/// let _ = EguiTextCommandSurfaceEditor::new(Provider);
/// ```
pub struct EguiTextCommandSurfaceEditor<Provider> {
    binding: HostProjectionBinding<Provider>,
}

#[derive(Debug)]
pub enum EguiTextCommandSurfaceEditorError<ProviderError, ForwarderError> {
    Binding(HostProjectionBindingError<ProviderError, ForwarderError>),
}
pub type EguiTextCommandSurfaceEditorResult<ProviderError, ForwarderError> =
    Result<KucRootBindingReceipt, EguiTextCommandSurfaceEditorError<ProviderError, ForwarderError>>;

#[cfg(feature = "storybook-artifacts")]
pub type EguiTextCommandSurfaceEditorArtifactResult<ProviderError, ForwarderError> = Result<
    (
        katana_ui_core::egui::OpaqueRootArtifactReceipt,
        KucRootBindingReceipt,
    ),
    EguiTextCommandSurfaceEditorError<ProviderError, ForwarderError>,
>;

impl<Provider> EguiTextCommandSurfaceEditor<Provider>
where
    Provider: HostProjectionProvider<Lease = EguiTextCommandSurfaceHostProjectionLease>,
{
    pub fn new(
        provider: Provider,
    ) -> Result<Self, EguiTextCommandSurfaceEditorError<Provider::Error, std::convert::Infallible>>
    {
        let binding = HostProjectionBinding::new(provider)
            .map_err(EguiTextCommandSurfaceEditorError::Binding)?;
        Ok(Self { binding })
    }
    pub fn show<Forwarder>(
        &mut self,
        ui: &mut Ui,
        forwarder: &mut Forwarder,
    ) -> EguiTextCommandSurfaceEditorResult<Provider::Error, Forwarder::Error>
    where
        Forwarder: KucRootEventBatchForwarder,
    {
        self.binding
            .show_and_forward_once(ui, forwarder)
            .map_err(EguiTextCommandSurfaceEditorError::Binding)
    }
    pub fn show_with_opaque_frame<Forwarder, Callback>(
        &mut self,
        ui: &mut Ui,
        forwarder: &mut Forwarder,
        callback: Callback,
    ) -> EguiTextCommandSurfaceEditorResult<Provider::Error, Forwarder::Error>
    where
        Forwarder: KucRootEventBatchForwarder,
        Callback: FnOnce(&KucInteractionLocator),
    {
        self.binding
            .show_with_opaque_frame(ui, forwarder, callback)
            .map_err(EguiTextCommandSurfaceEditorError::Binding)
    }
    #[cfg(feature = "storybook-artifacts")]
    pub fn show_write_artifact<Forwarder>(
        &mut self,
        ui: &mut Ui,
        output_dir: &std::path::Path,
        stage_id: &str,
        forwarder: &mut Forwarder,
    ) -> crate::EguiTextCommandSurfaceEditorArtifactResult<Provider::Error, Forwarder::Error>
    where
        Forwarder: KucRootEventBatchForwarder,
    {
        self.binding
            .show_write_artifact_and_forward_once(ui, output_dir, stage_id, forwarder)
            .map_err(EguiTextCommandSurfaceEditorError::Binding)
    }
    #[cfg(feature = "storybook-artifacts")]
    pub fn show_write_artifact_with_opaque_frame<Forwarder, Callback>(
        &mut self,
        ui: &mut Ui,
        output_dir: &std::path::Path,
        stage_id: &str,
        forwarder: &mut Forwarder,
        callback: Callback,
    ) -> crate::EguiTextCommandSurfaceEditorArtifactResult<Provider::Error, Forwarder::Error>
    where
        Forwarder: KucRootEventBatchForwarder,
        Callback: FnOnce(&KucInteractionLocator),
    {
        self.binding
            .show_write_artifact_with_opaque_frame(ui, output_dir, stage_id, forwarder, callback)
            .map_err(EguiTextCommandSurfaceEditorError::Binding)
    }
}

#[cfg(test)]
#[path = "root_editor_tests.rs"]
mod tests;
