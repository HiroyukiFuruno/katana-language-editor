use crate::kuc_root_binding::{
    KucRootBinding, KucRootBindingError, KucRootBindingFactoryError, KucRootBindingReceipt,
};
use egui::Ui;
pub use katana_language_editor::{HostProjectionProvider, HostProjectionProviderError};
#[cfg(feature = "storybook-artifacts")]
use katana_ui_core::egui::OpaqueRootArtifactReceipt;
use katana_ui_core::egui::text_command_surface::EguiTextCommandSurfaceHostProjectionLease;
use katana_ui_core::egui::text_command_surface::KucInteractionLocator;
#[cfg(feature = "storybook-artifacts")]
use std::path::Path;

#[derive(Debug)]
pub enum HostProjectionBindingError<ProviderError, ForwarderError> {
    Provider(HostProjectionProviderError<ProviderError>),
    Retain(KucRootBindingFactoryError),
    Frame(KucRootBindingError<ForwarderError>),
}

pub type HostProjectionFrameResult<ProviderError, ForwarderError> =
    Result<KucRootBindingReceipt, HostProjectionBindingError<ProviderError, ForwarderError>>;

#[cfg(feature = "storybook-artifacts")]
pub type HostProjectionArtifactResult<ProviderError, ForwarderError> = Result<
    (OpaqueRootArtifactReceipt, KucRootBindingReceipt),
    HostProjectionBindingError<ProviderError, ForwarderError>,
>;

/// Couples a provider's one-shot tokens to one retained KUC root.
///
/// vendor型の制約はpublic editorだけでなくbindingでも維持する。
///
/// ```compile_fail
/// use katana_language_editor::HostProjectionProvider;
/// use katana_language_editor_egui::HostProjectionBinding;
/// fn wrong_lease<P: HostProjectionProvider<Lease = ()>>(provider: P) {
///     let _ = HostProjectionBinding::new(provider);
/// }
/// ```
pub struct HostProjectionBinding<Provider> {
    provider: Provider,
    binding: KucRootBinding,
}

impl<Provider> HostProjectionBinding<Provider>
where
    Provider: HostProjectionProvider<Lease = EguiTextCommandSurfaceHostProjectionLease>,
{
    pub fn new(
        mut provider: Provider,
    ) -> Result<Self, HostProjectionBindingError<Provider::Error, std::convert::Infallible>> {
        let lease = provider
            .retain_lease()
            .map_err(HostProjectionBindingError::Provider)?;
        let binding = KucRootBinding::new(lease).map_err(HostProjectionBindingError::Retain)?;
        Ok(Self { provider, binding })
    }

    pub fn show_and_forward_once<Forwarder>(
        &mut self,
        ui: &mut Ui,
        forwarder: &mut Forwarder,
    ) -> HostProjectionFrameResult<Provider::Error, Forwarder::Error>
    where
        Forwarder: katana_ui_core::egui::text_command_surface::KucRootEventBatchForwarder,
    {
        let lease = self
            .provider
            .synchronize_lease()
            .map_err(HostProjectionBindingError::Provider)?;
        self.binding
            .show_and_forward_once(lease, ui, forwarder)
            .map_err(HostProjectionBindingError::Frame)
    }

    pub fn show_with_opaque_frame<Forwarder, Callback>(
        &mut self,
        ui: &mut Ui,
        forwarder: &mut Forwarder,
        callback: Callback,
    ) -> HostProjectionFrameResult<Provider::Error, Forwarder::Error>
    where
        Forwarder: katana_ui_core::egui::text_command_surface::KucRootEventBatchForwarder,
        Callback: FnOnce(&KucInteractionLocator),
    {
        let lease = self
            .provider
            .synchronize_lease()
            .map_err(HostProjectionBindingError::Provider)?;
        self.binding
            .show_with_opaque_frame(lease, ui, forwarder, callback)
            .map_err(HostProjectionBindingError::Frame)
    }

    #[cfg(feature = "storybook-artifacts")]
    pub fn show_write_artifact_and_forward_once<Forwarder>(
        &mut self,
        ui: &mut Ui,
        output_dir: &Path,
        stage_id: &str,
        forwarder: &mut Forwarder,
    ) -> HostProjectionArtifactResult<Provider::Error, Forwarder::Error>
    where
        Forwarder: katana_ui_core::egui::text_command_surface::KucRootEventBatchForwarder,
    {
        let lease = self
            .provider
            .synchronize_lease()
            .map_err(HostProjectionBindingError::Provider)?;
        self.binding
            .show_write_artifact_and_forward_once(lease, ui, output_dir, stage_id, forwarder)
            .map_err(HostProjectionBindingError::Frame)
    }

    #[cfg(feature = "storybook-artifacts")]
    pub fn show_write_artifact_with_opaque_frame<Forwarder, Callback>(
        &mut self,
        ui: &mut Ui,
        output_dir: &Path,
        stage_id: &str,
        forwarder: &mut Forwarder,
        callback: Callback,
    ) -> HostProjectionArtifactResult<Provider::Error, Forwarder::Error>
    where
        Forwarder: katana_ui_core::egui::text_command_surface::KucRootEventBatchForwarder,
        Callback: FnOnce(&KucInteractionLocator),
    {
        let lease = self
            .provider
            .synchronize_lease()
            .map_err(HostProjectionBindingError::Provider)?;
        self.binding
            .show_write_artifact_with_opaque_frame(
                lease, ui, output_dir, stage_id, forwarder, callback,
            )
            .map_err(HostProjectionBindingError::Frame)
    }
}

#[cfg(test)]
#[path = "host_projection_provider_tests.rs"]
mod tests;
