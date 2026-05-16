//! Floem 実装 crate の skeleton。
//!
//! v0.1.0 では中立 API（neutral API）と git 固定の依存関係（dependency）が
//! 同時にコンパイルできることだけを保証する。本実装は v0.2.x で追加する。

use katana_language_editor::{CursorPosition, EditorEvent, LanguageEditor, TextContent};

#[derive(Default)]
pub struct FloemLanguageEditor {
    content: TextContent,
    pending_events: Vec<EditorEvent>,
}

impl FloemLanguageEditor {
    pub fn new() -> Self {
        Self::default()
    }
}

impl LanguageEditor for FloemLanguageEditor {
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
