//! katana-language-editor: vendor-neutral language editor interface.
//!
//! Neutral trait surface and DTO types. No dependency on egui or any UI
//! framework. The egui MVP implementation lives in
//! `katana-language-editor-egui`.
//!
//! When KatanA migrates away from egui, only the `-egui` crate changes;
//! KatanA's dependency on this crate stays unchanged.

pub mod types;
pub use types::{
    CursorPosition, EditorConfig, EditorDiagnostics, EditorError, EditorEvent, EditorOutput,
    LanguageEditor, Selection, TextContent,
};
