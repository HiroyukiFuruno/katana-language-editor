use super::{KucRootBinding, KucRootBindingError, KucRootBindingReceipt, binding_receipt};
use katana_ui_core::egui::text_command_surface::{
    EguiTextCommandSurfaceHostProjectionLease, KucRootEventBatchForwarder,
};
use katana_ui_core::egui::{OpaqueRootArtifactReceipt, OpaqueRootArtifactReceiptWriter};
use std::path::Path;

impl KucRootBinding {
    pub fn show_write_artifact_and_forward_once<Forwarder>(
        &mut self,
        lease: EguiTextCommandSurfaceHostProjectionLease,
        ui: &mut egui::Ui,
        output_dir: &Path,
        stage_id: &str,
        forwarder: &mut Forwarder,
    ) -> Result<
        (OpaqueRootArtifactReceipt, KucRootBindingReceipt),
        KucRootBindingError<Forwarder::Error>,
    >
    where
        Forwarder: KucRootEventBatchForwarder,
    {
        self.root
            .synchronize_with_lease(lease)
            .map_err(KucRootBindingError::Synchronize)?;
        self.write_current_artifact_and_forward_once(ui, output_dir, stage_id, forwarder)
    }

    pub fn show_current_write_artifact_and_forward_once<Forwarder>(
        &mut self,
        ui: &mut egui::Ui,
        output_dir: &Path,
        stage_id: &str,
        forwarder: &mut Forwarder,
    ) -> Result<
        (OpaqueRootArtifactReceipt, KucRootBindingReceipt),
        KucRootBindingError<Forwarder::Error>,
    >
    where
        Forwarder: KucRootEventBatchForwarder,
    {
        self.write_current_artifact_and_forward_once(ui, output_dir, stage_id, forwarder)
    }

    pub fn show_current_write_artifact_with_opaque_frame<Forwarder, Callback>(
        &mut self,
        ui: &mut egui::Ui,
        output_dir: &Path,
        stage_id: &str,
        forwarder: &mut Forwarder,
        callback: Callback,
    ) -> Result<
        (OpaqueRootArtifactReceipt, KucRootBindingReceipt),
        KucRootBindingError<Forwarder::Error>,
    >
    where
        Forwarder: KucRootEventBatchForwarder,
        Callback: FnOnce(&katana_ui_core::egui::text_command_surface::KucInteractionLocator),
    {
        let frame = self.root.show(ui).map_err(KucRootBindingError::Render)?;
        let artifact = write_artifact(&frame, output_dir, stage_id)?;
        let forwarding = frame
            .forward_events_once(forwarder)
            .map_err(KucRootBindingError::Forward)?;
        callback(frame.interaction_locator());
        Ok((artifact, binding_receipt(frame.record(), &forwarding)))
    }

    pub fn show_write_artifact_with_opaque_frame<Forwarder, Callback>(
        &mut self,
        lease: EguiTextCommandSurfaceHostProjectionLease,
        ui: &mut egui::Ui,
        output_dir: &Path,
        stage_id: &str,
        forwarder: &mut Forwarder,
        callback: Callback,
    ) -> Result<
        (OpaqueRootArtifactReceipt, KucRootBindingReceipt),
        KucRootBindingError<Forwarder::Error>,
    >
    where
        Forwarder: KucRootEventBatchForwarder,
        Callback: FnOnce(&katana_ui_core::egui::text_command_surface::KucInteractionLocator),
    {
        self.root
            .synchronize_with_lease(lease)
            .map_err(KucRootBindingError::Synchronize)?;
        self.show_current_write_artifact_with_opaque_frame(
            ui, output_dir, stage_id, forwarder, callback,
        )
    }

    fn write_current_artifact_and_forward_once<Forwarder>(
        &mut self,
        ui: &mut egui::Ui,
        output_dir: &Path,
        stage_id: &str,
        forwarder: &mut Forwarder,
    ) -> Result<
        (OpaqueRootArtifactReceipt, KucRootBindingReceipt),
        KucRootBindingError<Forwarder::Error>,
    >
    where
        Forwarder: KucRootEventBatchForwarder,
    {
        let frame = self.root.show(ui).map_err(KucRootBindingError::Render)?;
        let artifact = write_artifact(&frame, output_dir, stage_id)?;
        let forwarding = frame
            .forward_events_once(forwarder)
            .map_err(KucRootBindingError::Forward)?;
        Ok((artifact, binding_receipt(frame.record(), &forwarding)))
    }
}

fn write_artifact<ForwarderError>(
    frame: &katana_ui_core::egui::text_command_surface::EguiTextCommandSurfaceHostRootFrame,
    output_dir: &Path,
    stage_id: &str,
) -> Result<OpaqueRootArtifactReceipt, KucRootBindingError<ForwarderError>> {
    OpaqueRootArtifactReceiptWriter::new()
        .write(frame, output_dir, stage_id)
        .map_err(KucRootBindingError::Artifact)
}
