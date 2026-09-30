//! katana-language-editor: vendor-neutral language editor interface.
//!
//! Neutral trait surface and DTO types. No dependency on egui or any UI
//! framework. A KUC-backed egui integration lives in
//! `katana-language-editor-egui`; this crate does not claim full editor parity.
//!
//! When KatanA migrates away from egui, only the `-egui` crate changes;
//! KatanA's dependency on this crate stays unchanged.
//!
//! ```compile_fail
//! use katana_language_editor::Typography;
//! ```
//!
//! ```compile_fail
//! use katana_language_editor::Spacing;
//! ```
//!
//! ```compile_fail
//! use katana_language_editor::EditorSettings;
//! ```
//!
//! ```compile_fail
//! use katana_language_editor::AutosavePolicy;
//! ```
//!
//! ```compile_fail
//! use katana_language_editor::ShortcutMap;
//! ```
//!
//! ```compile_fail
//! use katana_language_editor::ShortcutBinding;
//! ```
//!
//! ```compile_fail
//! use katana_language_editor::KeyBinding;
//! ```
//!
//! ```compile_fail
//! use katana_language_editor::KeyModifier;
//! ```
//!
//! ```compile_fail
//! use katana_language_editor::SemanticAction;
//! ```
//!
//! ```compile_fail
//! use katana_language_editor::Rgba;
//! ```
//!
//! ```compile_fail
//! use katana_language_editor::ColorTokens;
//! ```
//!
//! ```compile_fail
//! use katana_language_editor::ColorTokensInput;
//! ```
//!
//! ```compile_fail
//! use katana_language_editor::Theme;
//! ```
//!
//! ```compile_fail
//! use katana_language_editor::EditorTheme;
//! ```

mod accessibility;
mod actions;
mod clipboard;
mod clipboard_paste_control;
mod config;
mod content;
mod controls;
mod document_state;
mod document_state_report;
mod editor;
mod error;
mod events;
mod highlight;
mod host_projection;
mod localization;
mod position;
pub mod types;

pub use accessibility::{AccessibilityConfig, EditorAccessibility, MotionPreference};
pub use actions::{EditorSplitDirection, EditorViewMode};
pub use clipboard::{
    ClipboardBackend, ClipboardPasteToken, EditorClipboardPasteRejection,
    EditorClipboardPasteReport, EditorClipboardPasteRequest, EditorClipboardPasteResolution,
    EditorClipboardPasteStatus,
};
pub use clipboard_paste_control::EditorClipboardPasteControl;
pub use config::{EditorConfig, EditorConfigInput};
pub use content::TextContent;
pub use controls::{
    EditorClipboardControl, EditorCursorRestoreControl, EditorDocumentStateControl,
    EditorHistoryControl, EditorSelectionControl, EditorWriteAccess, WriteHandle,
};
pub use document_state::{
    EditorDocumentIdentity, EditorDocumentState, EditorDocumentUpdate, EditorDocumentUpdateOrigin,
    EditorDocumentUpdateReport, EditorExternalUndoRecord,
};
pub use editor::LanguageEditor;
pub use error::{EditorError, EditorResult};
pub use events::{EditorEvent, EditorOutput};
pub use highlight::{
    HighlightedSpan, HighlightedText, NoopSyntaxHighlighter, SyntaxHighlighter, TokenKind,
};
pub use host_projection::{HostProjectionProvider, HostProjectionProviderError};
pub use localization::{
    EditorStrings, EditorStringsInput, Locale, Strings, StringsInput, TextDirection,
};
pub use position::{CursorPosition, Selection, TextOffset, TextRange, VisibleRange};
