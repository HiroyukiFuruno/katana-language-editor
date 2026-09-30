use super::root_dispatch_evidence::{
    StorybookDispatchEvidenceError, StorybookGenericRootForwarder,
};
use super::{StorybookRootError, StorybookRootFrame};
use katana_language_editor_egui::{
    EguiTextCommandSurfaceEditor, EguiTextCommandSurfaceEditorArtifactResult,
    HostProjectionProvider, KucRootBindingReceipt,
};
use katana_ui_core::egui::text_command_surface::EguiTextCommandSurfaceHostProjectionLease;
use katana_ui_core::egui::text_command_surface::KucRootEventBatchForwarder;

type ArtifactFrameResult<Provider, Forwarder> = EguiTextCommandSurfaceEditorArtifactResult<
    <Provider as HostProjectionProvider>::Error,
    <Forwarder as KucRootEventBatchForwarder>::Error,
>;

pub(super) fn show_editor_frame<Provider, Forwarder>(
    editor: &mut EguiTextCommandSurfaceEditor<Provider>,
    ui: &mut egui::Ui,
    output_dir: &std::path::Path,
    stage_id: &str,
    forwarder: &mut Forwarder,
    callback: &mut dyn FnMut(&katana_ui_core::egui::text_command_surface::KucInteractionLocator),
) -> ArtifactFrameResult<Provider, Forwarder>
where
    Provider: HostProjectionProvider<Lease = EguiTextCommandSurfaceHostProjectionLease>,
    Forwarder: KucRootEventBatchForwarder,
{
    editor.show_write_artifact_with_opaque_frame(ui, output_dir, stage_id, forwarder, callback)
}

pub(super) fn build_root_frame(
    artifact_receipt: katana_ui_core::egui::OpaqueRootArtifactReceipt,
    receipt: KucRootBindingReceipt,
    forwarder: StorybookGenericRootForwarder,
) -> Result<StorybookRootFrame, StorybookRootError> {
    let evidence = forwarder.into_evidence().map_err(|error| match error {
        StorybookDispatchEvidenceError::MissingDispatchReceipt => {
            StorybookRootError::MissingDispatchReceipt
        }
    })?;
    Ok(StorybookRootFrame {
        artifact_receipt,
        receipt,
        dispatch_receipt: *evidence.dispatch_receipt(),
        class_dispatch_records: evidence.class_dispatch_records().to_vec(),
    })
}
