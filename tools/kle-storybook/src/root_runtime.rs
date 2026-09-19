use crate::host_types::StorybookRootFrame;
#[path = "root_dispatch_evidence.rs"]
mod root_dispatch_evidence;
#[path = "root_runtime_frame.rs"]
mod root_runtime_frame;
use katana_language_editor_egui::{
    EguiTextCommandSurfaceEditor, HostProjectionProvider, KucRootBindingReceipt,
};
use katana_ui_core::egui::text_command_surface::EguiTextCommandSurfaceHostProjectionLease;
use root_dispatch_evidence::{StorybookDispatchEvidence, StorybookGenericRootForwarder};
use std::path::Path;
#[derive(Debug)]
pub enum StorybookRootError {
    Binding(String),
    MissingDispatchReceipt,
    InvalidInteractiveEvidence(&'static str),
}
impl std::fmt::Display for StorybookRootError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Binding(error) => formatter.write_str(error),
            Self::MissingDispatchReceipt => {
                formatter.write_str("generic KUC dispatch receipt was not retained")
            }
            Self::InvalidInteractiveEvidence(reason) => formatter.write_str(reason),
        }
    }
}
impl std::error::Error for StorybookRootError {}

pub trait StorybookProjection {
    fn show_interactive(
        &mut self,
        ui: &mut egui::Ui,
    ) -> Result<KucRootBindingReceipt, StorybookRootError>;

    fn show_write_artifact_and_forward_once(
        &mut self,
        ui: &mut egui::Ui,
        output_dir: &Path,
        stage_id: &str,
        callback: &mut dyn FnMut(
            &katana_ui_core::egui::text_command_surface::KucInteractionLocator,
        ),
    ) -> Result<StorybookRootFrame, StorybookRootError>;
}

pub struct InjectedProjection<Provider> {
    editor: EguiTextCommandSurfaceEditor<Provider>,
}

impl<Provider> InjectedProjection<Provider>
where
    Provider: HostProjectionProvider<Lease = EguiTextCommandSurfaceHostProjectionLease>,
    Provider::Error: std::fmt::Debug,
{
    pub(crate) fn new(provider: Provider) -> Result<Self, StorybookRootError> {
        EguiTextCommandSurfaceEditor::new(provider)
            .map(|editor| Self { editor })
            .map_err(|error| StorybookRootError::Binding(format!("KLE provider failed: {error:?}")))
    }
}

impl<Provider> StorybookProjection for InjectedProjection<Provider>
where
    Provider: HostProjectionProvider<Lease = EguiTextCommandSurfaceHostProjectionLease>,
    Provider::Error: std::fmt::Debug,
{
    fn show_interactive(
        &mut self,
        ui: &mut egui::Ui,
    ) -> Result<KucRootBindingReceipt, StorybookRootError> {
        let mut forwarder = StorybookGenericRootForwarder::default();
        let receipt = self.editor.show(ui, &mut forwarder).map_err(|error| {
            StorybookRootError::Binding(format!("KLE public editor frame failed: {error:?}"))
        })?;
        validate_interactive_evidence(&receipt, forwarder)?;
        Ok(receipt)
    }

    fn show_write_artifact_and_forward_once(
        &mut self,
        ui: &mut egui::Ui,
        output_dir: &Path,
        stage_id: &str,
        callback: &mut dyn FnMut(
            &katana_ui_core::egui::text_command_surface::KucInteractionLocator,
        ),
    ) -> Result<StorybookRootFrame, StorybookRootError> {
        let mut forwarder = StorybookGenericRootForwarder::default();
        root_runtime_frame::show_editor_frame(
            &mut self.editor,
            ui,
            output_dir,
            stage_id,
            &mut forwarder,
            callback,
        )
        .map(|(artifact, receipt)| {
            root_runtime_frame::build_root_frame(artifact, receipt, forwarder)
        })
        .map_err(|error| {
            StorybookRootError::Binding(format!("KLE public editor frame failed: {error:?}"))
        })?
        .map_err(|error| {
            StorybookRootError::Binding(format!("generic KUC evidence failed: {error}"))
        })
    }
}

fn validate_interactive_evidence(
    receipt: &KucRootBindingReceipt,
    forwarder: StorybookGenericRootForwarder,
) -> Result<(), StorybookRootError> {
    let evidence = forwarder
        .into_evidence()
        .map_err(|_| StorybookRootError::MissingDispatchReceipt)?;
    validate_interactive_dispatch(receipt, &evidence)?;
    if !receipt.consumed_once() {
        return Err(StorybookRootError::InvalidInteractiveEvidence(
            "generic KUC root event batch was not consumed once",
        ));
    }
    Ok(())
}

fn validate_interactive_dispatch(
    receipt: &KucRootBindingReceipt,
    evidence: &StorybookDispatchEvidence,
) -> Result<(), StorybookRootError> {
    let dispatched_by_root = evidence
        .dispatch_receipt()
        .class_dispatches()
        .iter()
        .map(|record| (record.child_class, record.event_count))
        .collect::<Vec<_>>();
    if dispatched_by_root != evidence.class_dispatch_records() {
        return Err(StorybookRootError::InvalidInteractiveEvidence(
            "generic KUC dispatch receipt and Storybook record diverged",
        ));
    }
    let forwarded = evidence
        .class_dispatch_records()
        .iter()
        .map(|(_, count)| count)
        .sum::<usize>();
    if forwarded > receipt.event_cardinality() {
        return Err(StorybookRootError::InvalidInteractiveEvidence(
            "generic KUC dispatch count exceeds the closed root event cardinality",
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "root_runtime_interactive_tests.rs"]
mod interactive_tests;
