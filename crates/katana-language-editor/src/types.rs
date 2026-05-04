use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextContent {
    pub text: String,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CursorPosition {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Selection {
    pub start: CursorPosition,
    pub end: CursorPosition,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EditorConfig {
    pub font_size: Option<f32>,
    pub line_numbers: bool,
    pub word_wrap: bool,
    pub tab_size: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditorEvent {
    ContentChanged(TextContent),
    CursorMoved(CursorPosition),
    SelectionChanged(Selection),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EditorDiagnostics {
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorOutput {
    pub cursor: CursorPosition,
    pub selection: Option<Selection>,
    pub diagnostics: EditorDiagnostics,
}

#[derive(Debug, Error)]
pub enum EditorError {
    #[error("not implemented")]
    NotImplemented,
    #[error("editor error: {0}")]
    Internal(String),
}

/// Vendor-neutral language editor trait.
///
/// KatanA depends on this trait. egui and future custom UI implementations
/// satisfy it without leaking framework types into KatanA.
pub trait LanguageEditor {
    fn content(&self) -> &TextContent;
    fn set_content(&mut self, content: TextContent);
    fn cursor(&self) -> CursorPosition;
    fn poll_events(&mut self) -> Vec<EditorEvent>;
}
