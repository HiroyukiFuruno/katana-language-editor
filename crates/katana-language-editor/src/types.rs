//! Public type reexports for the neutral editor interface.
//!
//! ```compile_fail
//! use katana_language_editor::types::Typography;
//! ```
//!
//! ```compile_fail
//! use katana_language_editor::types::Spacing;
//! ```
//!
//! ```compile_fail
//! use katana_language_editor::types::EditorSettings;
//! ```
//!
//! ```compile_fail
//! use katana_language_editor::types::AutosavePolicy;
//! ```
//!
//! ```compile_fail
//! use katana_language_editor::types::ShortcutMap;
//! ```
//!
//! ```compile_fail
//! use katana_language_editor::types::ShortcutBinding;
//! ```
//!
//! ```compile_fail
//! use katana_language_editor::types::KeyBinding;
//! ```
//!
//! ```compile_fail
//! use katana_language_editor::types::KeyModifier;
//! ```
//!
//! ```compile_fail
//! use katana_language_editor::types::SemanticAction;
//! ```
//!
//! ```compile_fail
//! use katana_language_editor::types::Rgba;
//! ```
//!
//! ```compile_fail
//! use katana_language_editor::types::ColorTokens;
//! ```
//!
//! ```compile_fail
//! use katana_language_editor::types::ColorTokensInput;
//! ```
//!
//! ```compile_fail
//! use katana_language_editor::types::Theme;
//! ```
//!
//! ```compile_fail
//! use katana_language_editor::types::EditorTheme;
//! ```

pub use crate::{
    AccessibilityConfig, ClipboardBackend, CursorPosition, EditorAccessibility,
    EditorClipboardControl, EditorConfig, EditorConfigInput, EditorCursorRestoreControl,
    EditorDocumentIdentity, EditorDocumentState, EditorDocumentStateControl, EditorDocumentUpdate,
    EditorDocumentUpdateOrigin, EditorDocumentUpdateReport, EditorError, EditorEvent,
    EditorExternalUndoRecord, EditorHistoryControl, EditorOutput, EditorResult,
    EditorSelectionControl, EditorStrings, EditorStringsInput, EditorWriteAccess, HighlightedSpan,
    HighlightedText, LanguageEditor, Locale, MotionPreference, NoopSyntaxHighlighter, Selection,
    Strings, StringsInput, SyntaxHighlighter, TextContent, TextDirection, TextOffset, TextRange,
    TokenKind, VisibleRange, WriteHandle,
};
