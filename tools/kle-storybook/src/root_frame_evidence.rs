use katana_language_editor_egui::KucRootBindingReceipt;
use katana_ui_core::egui::OpaqueRootArtifactReceipt;
use katana_ui_core::egui::text_command_surface::{
    EguiTextCommandSurfaceRootEventChildClass, EguiTextCommandSurfaceRootEventDispatchReceipt,
};

#[derive(Debug)]
pub(crate) struct StorybookRootFrame {
    pub(crate) artifact_receipt: OpaqueRootArtifactReceipt,
    pub(crate) receipt: KucRootBindingReceipt,
    pub(crate) dispatch_receipt: EguiTextCommandSurfaceRootEventDispatchReceipt,
    pub(crate) class_dispatch_records: Vec<(EguiTextCommandSurfaceRootEventChildClass, usize)>,
}

impl StorybookRootFrame {
    pub(crate) fn validate_live_evidence(&self) -> Result<(), String> {
        self.validate_root_receipt_evidence()?;
        self.validate_dispatch_receipt_coherence()
    }

    fn validate_root_receipt_evidence(&self) -> Result<(), String> {
        if self.artifact_receipt.stage_id().is_empty()
            || self.receipt.record_hash().is_empty()
            || self.receipt.paint_plan_hash().is_empty()
            || self.receipt.accessibility_snapshot_hash().is_empty()
            || !self.receipt.consumed_once()
        {
            return Err("KUC root evidence is incomplete".to_string());
        }
        Ok(())
    }

    fn validate_dispatch_receipt_coherence(&self) -> Result<(), String> {
        let receipt_records = self
            .dispatch_receipt
            .class_dispatches()
            .iter()
            .map(|record| (record.child_class, record.event_count))
            .collect::<Vec<_>>();
        if receipt_records != self.class_dispatch_records {
            return Err("generic KUC dispatch receipt and Storybook record diverged".to_string());
        }
        let dispatched_events = self
            .class_dispatch_records
            .iter()
            .map(|(_, count)| count)
            .sum::<usize>();
        if dispatched_events > self.receipt.event_cardinality() {
            return Err(
                "public KUC dispatch count exceeds the closed root event cardinality".to_string(),
            );
        }
        Ok(())
    }
}
