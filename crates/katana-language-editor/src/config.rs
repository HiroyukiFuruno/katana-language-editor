//! The generic presentation policy is owned by the KUC root.
//!
//! ```compile_fail
//! # use katana_language_editor::EditorConfig;
//! # fn reject(config: EditorConfig) {
//! let _ = config.theme;
//! # }
//! ```
//!
//! ```compile_fail
//! # use katana_language_editor::EditorConfig;
//! # fn reject(config: EditorConfig) {
//! let _ = config.strings;
//! # }
//! ```
//!
//! ```compile_fail
//! # use katana_language_editor::EditorConfig;
//! # fn reject(config: EditorConfig) {
//! let _ = config.locale;
//! # }
//! ```
//!
//! ```compile_fail
//! # use katana_language_editor::EditorConfigInput;
//! # fn reject(input: EditorConfigInput) {
//! let _ = input.theme;
//! # }
//! ```
//!
//! ```compile_fail
//! # use katana_language_editor::EditorConfigInput;
//! # fn reject(input: EditorConfigInput) {
//! let _ = input.strings;
//! # }
//! ```
//!
//! ```compile_fail
//! # use katana_language_editor::EditorConfigInput;
//! # fn reject(input: EditorConfigInput) {
//! let _ = input.locale;
//! # }
//! ```
//!
//! ```
//! # use katana_language_editor::{EditorConfig, EditorConfigInput};
//! fn accept(input: EditorConfigInput) -> EditorConfig {
//!     EditorConfig::new(input)
//! }
//! ```

use crate::{ClipboardBackend, SyntaxHighlighter};
use std::sync::Arc;

pub struct EditorConfigInput {
    pub syntax_highlighter: Arc<dyn SyntaxHighlighter>,
    pub clipboard: Arc<dyn ClipboardBackend>,
}

#[derive(Clone)]
#[non_exhaustive]
pub struct EditorConfig {
    pub syntax_highlighter: Arc<dyn SyntaxHighlighter>,
    pub clipboard: Arc<dyn ClipboardBackend>,
}

impl EditorConfig {
    pub fn new(input: EditorConfigInput) -> Self {
        Self {
            syntax_highlighter: input.syntax_highlighter,
            clipboard: input.clipboard,
        }
    }
}
