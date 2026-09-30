#[cfg(feature = "storybook-artifacts")]
use katana_ui_core::egui::OpaqueRootArtifactReceiptError;
use katana_ui_core::egui::text_command_surface::KucInteractionLocator;
use katana_ui_core::egui::text_command_surface::{
    EguiTextCommandSurfaceHostProjectionLease, EguiTextCommandSurfaceHostRoot,
    EguiTextCommandSurfaceHostRootRecord, EguiTextCommandSurfaceHostRootRecordDimensions,
    EguiTextCommandSurfaceRootEventBatchForwardError,
    EguiTextCommandSurfaceRootEventForwardingReceipt, EguiTextCommandSurfaceRootFactory,
    EguiTextCommandSurfaceRootFactoryError, KucRootEventBatchForwarder,
};

/// KLE binding that retains only the opaque KUC host root.
///
/// leaseを再利用する経路は、KUCへ渡した時点の所有権移動で拒否する。
///
/// ```compile_fail
/// use katana_language_editor_egui::KucRootBinding;
/// use katana_ui_core::egui::text_command_surface::EguiTextCommandSurfaceHostProjectionLease;
/// fn reuse(lease: EguiTextCommandSurfaceHostProjectionLease) {
///     let _ = KucRootBinding::new(lease);
///     let _ = KucRootBinding::new(lease);
/// }
/// ```
///
/// ```compile_fail
/// use katana_ui_core::egui::text_command_surface::EguiTextCommandSurfaceHostProjectionLease;
/// fn duplicate(lease: EguiTextCommandSurfaceHostProjectionLease) {
///     let _ = lease.clone();
/// }
/// ```
pub struct KucRootBinding {
    root: EguiTextCommandSurfaceHostRoot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KucRootBindingReceipt {
    root_identity: String,
    presentation_revision: u64,
    state_revision: u64,
    dimensions: EguiTextCommandSurfaceHostRootRecordDimensions,
    paint_plan_hash: String,
    record_hash: String,
    accessibility_snapshot_hash: String,
    correlation_fingerprint: String,
    event_batch_fingerprint: String,
    event_cardinality: usize,
    consumed_once: bool,
}

#[path = "kuc_root_binding_receipt.rs"]
mod receipt;

#[cfg(feature = "storybook-artifacts")]
#[path = "kuc_root_binding/artifact.rs"]
mod artifact;

#[derive(Debug)]
pub enum KucRootBindingFactoryError {
    Factory(EguiTextCommandSurfaceRootFactoryError),
}

impl From<EguiTextCommandSurfaceRootFactoryError> for KucRootBindingFactoryError {
    fn from(value: EguiTextCommandSurfaceRootFactoryError) -> Self {
        Self::Factory(value)
    }
}

#[derive(Debug)]
pub enum KucRootBindingError<ForwarderError> {
    Synchronize(EguiTextCommandSurfaceRootFactoryError),
    Render(EguiTextCommandSurfaceRootFactoryError),
    #[cfg(feature = "storybook-artifacts")]
    Artifact(OpaqueRootArtifactReceiptError),
    Forward(EguiTextCommandSurfaceRootEventBatchForwardError<ForwarderError>),
}

impl KucRootBinding {
    pub fn new(
        initial_lease: EguiTextCommandSurfaceHostProjectionLease,
    ) -> Result<Self, KucRootBindingFactoryError> {
        let root = EguiTextCommandSurfaceRootFactory::new().retain_with_lease(initial_lease)?;
        Ok(Self { root })
    }

    pub fn show_and_forward_once<Forwarder>(
        &mut self,
        lease: EguiTextCommandSurfaceHostProjectionLease,
        ui: &mut egui::Ui,
        forwarder: &mut Forwarder,
    ) -> Result<KucRootBindingReceipt, KucRootBindingError<Forwarder::Error>>
    where
        Forwarder: KucRootEventBatchForwarder,
    {
        self.root
            .synchronize_with_lease(lease)
            .map_err(KucRootBindingError::Synchronize)?;
        let frame = self.root.show(ui).map_err(KucRootBindingError::Render)?;
        let record = frame.record();
        let forwarding = frame
            .forward_events_once(forwarder)
            .map_err(KucRootBindingError::Forward)?;
        Ok(binding_receipt(record, &forwarding))
    }

    pub fn show_current_and_forward_once<Forwarder>(
        &mut self,
        ui: &mut egui::Ui,
        forwarder: &mut Forwarder,
    ) -> Result<KucRootBindingReceipt, KucRootBindingError<Forwarder::Error>>
    where
        Forwarder: KucRootEventBatchForwarder,
    {
        let frame = self.root.show(ui).map_err(KucRootBindingError::Render)?;
        let forwarding = frame
            .forward_events_once(forwarder)
            .map_err(KucRootBindingError::Forward)?;
        Ok(binding_receipt(frame.record(), &forwarding))
    }

    pub fn show_with_opaque_frame<Forwarder, Callback>(
        &mut self,
        lease: EguiTextCommandSurfaceHostProjectionLease,
        ui: &mut egui::Ui,
        forwarder: &mut Forwarder,
        callback: Callback,
    ) -> Result<KucRootBindingReceipt, KucRootBindingError<Forwarder::Error>>
    where
        Forwarder: KucRootEventBatchForwarder,
        Callback: FnOnce(&KucInteractionLocator),
    {
        self.root
            .synchronize_with_lease(lease)
            .map_err(KucRootBindingError::Synchronize)?;
        let frame = self.root.show(ui).map_err(KucRootBindingError::Render)?;
        let record = frame.record();
        let forwarding = frame
            .forward_events_once(forwarder)
            .map_err(KucRootBindingError::Forward)?;
        callback(frame.interaction_locator());
        Ok(binding_receipt(record, &forwarding))
    }
}

fn binding_receipt(
    record: &EguiTextCommandSurfaceHostRootRecord,
    forwarding: &EguiTextCommandSurfaceRootEventForwardingReceipt,
) -> KucRootBindingReceipt {
    KucRootBindingReceipt {
        root_identity: record.identity().to_owned(),
        presentation_revision: record.presentation_revision(),
        state_revision: record.state_revision(),
        dimensions: record.dimensions(),
        paint_plan_hash: record.paint_plan_hash().to_owned(),
        record_hash: record.record_hash().to_owned(),
        accessibility_snapshot_hash: record.accessibility_snapshot_hash().to_owned(),
        correlation_fingerprint: forwarding.correlation_fingerprint().to_owned(),
        event_batch_fingerprint: forwarding.event_batch_fingerprint().to_owned(),
        event_cardinality: forwarding.event_cardinality(),
        consumed_once: forwarding.consumed_once(),
    }
}

#[path = "kuc_root_binding_source_tests.rs"]
mod source_contract;
