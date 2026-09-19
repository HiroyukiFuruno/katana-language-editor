use katana_ui_core::egui::text_command_surface::{
    EguiTextCommandSurfaceRootEventChildClass, EguiTextCommandSurfaceRootEventDispatchReceipt,
    EguiTextCommandSurfaceRootEventTransport, KucRootEventBatchDispatcher,
    KucRootEventBatchForwarder,
};
use katana_ui_core::molecule::command_chrome::{
    CommandChromeSearchEvent, CommandChromeToolbarEvent, FloatingCommandToolbarEvent,
};
use katana_ui_core::molecule::selection::ContextMenuEvent;
use katana_ui_core::text_surface::TextSurfaceEvent;
use std::convert::Infallible;

#[derive(Debug)]
pub(crate) enum StorybookDispatchEvidenceError {
    MissingDispatchReceipt,
}

#[derive(Debug, Default)]
pub(crate) struct StorybookGenericRootForwarder {
    dispatch_receipt: Option<EguiTextCommandSurfaceRootEventDispatchReceipt>,
    class_dispatch_records: Vec<(EguiTextCommandSurfaceRootEventChildClass, usize)>,
}

#[derive(Debug)]
pub(crate) struct StorybookDispatchEvidence {
    dispatch_receipt: EguiTextCommandSurfaceRootEventDispatchReceipt,
    class_dispatch_records: Vec<(EguiTextCommandSurfaceRootEventChildClass, usize)>,
}

impl StorybookDispatchEvidence {
    pub(crate) fn dispatch_receipt(&self) -> &EguiTextCommandSurfaceRootEventDispatchReceipt {
        &self.dispatch_receipt
    }

    pub(crate) fn class_dispatch_records(
        &self,
    ) -> &[(EguiTextCommandSurfaceRootEventChildClass, usize)] {
        &self.class_dispatch_records
    }
}

impl StorybookGenericRootForwarder {
    pub(crate) fn into_evidence(
        self,
    ) -> Result<StorybookDispatchEvidence, StorybookDispatchEvidenceError> {
        let dispatch_receipt = self
            .dispatch_receipt
            .ok_or(StorybookDispatchEvidenceError::MissingDispatchReceipt)?;
        Ok(StorybookDispatchEvidence {
            dispatch_receipt,
            class_dispatch_records: self.class_dispatch_records,
        })
    }

    fn retain_closed_class_dispatches(
        &mut self,
        receipt: EguiTextCommandSurfaceRootEventDispatchReceipt,
    ) {
        self.class_dispatch_records = receipt
            .class_dispatches()
            .iter()
            .map(|record| (record.child_class, record.event_count))
            .collect();
    }
}

impl KucRootEventBatchDispatcher for StorybookGenericRootForwarder {
    type Error = Infallible;

    fn dispatch_text_events(&mut self, _events: Vec<TextSurfaceEvent>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn dispatch_toolbar_events(
        &mut self,
        _events: Vec<CommandChromeToolbarEvent>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn dispatch_floating_events(
        &mut self,
        _events: Vec<FloatingCommandToolbarEvent>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn dispatch_search_events(
        &mut self,
        _events: Vec<CommandChromeSearchEvent>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn dispatch_context_menu_events(
        &mut self,
        _events: Vec<ContextMenuEvent>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl KucRootEventBatchForwarder for StorybookGenericRootForwarder {
    type Error = String;

    fn forward_root_event_batch(
        &mut self,
        transport: EguiTextCommandSurfaceRootEventTransport,
    ) -> Result<(), Self::Error> {
        let receipt = transport
            .dispatch_once(self)
            .map_err(|error| format!("generic KUC root dispatch failed: {error:?}"))?;
        self.retain_closed_class_dispatches(receipt);
        self.dispatch_receipt = Some(receipt);
        Ok(())
    }
}
