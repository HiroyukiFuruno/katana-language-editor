//! katana-language-editor-egui: egui implementation of LanguageEditor.
//!
//! MVP backend using egui TextEdit. KatanA wires in this implementation at
//! startup. Future: replaced by custom input surface (x-x-x-native-input-surface)
//! without touching KatanA's interface dependency.
//!
//! Status: scaffolding. Full implementation migrated from KatanA v0.27.0.

use katana_language_editor::{
    CursorPosition, EditorConfig, EditorEvent, LanguageEditor, TextContent,
};

pub struct EguiLanguageEditor {
    content: TextContent,
    config: EditorConfig,
    pending_events: Vec<EditorEvent>,
}

impl EguiLanguageEditor {
    pub fn new(config: EditorConfig) -> Self {
        Self {
            content: TextContent::default(),
            config,
            pending_events: Vec::new(),
        }
    }

    /// Draw the editor into an egui Ui. KatanA calls this during the MVP phase.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let response = ui.text_edit_multiline(&mut self.content.text);
        if response.changed() {
            self.pending_events
                .push(EditorEvent::ContentChanged(self.content.clone()));
        }
    }
}

impl LanguageEditor for EguiLanguageEditor {
    fn content(&self) -> &TextContent {
        &self.content
    }

    fn set_content(&mut self, content: TextContent) {
        self.content = content;
    }

    fn cursor(&self) -> CursorPosition {
        CursorPosition::default()
    }

    fn poll_events(&mut self) -> Vec<EditorEvent> {
        std::mem::take(&mut self.pending_events)
    }
}
