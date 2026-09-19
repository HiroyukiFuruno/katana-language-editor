use thiserror::Error;

pub type EditorResult<T> = Result<T, EditorError>;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EditorError {
    #[error("unsupported editor capability: {0}")]
    Unsupported(String),
    #[error("editor is read-only")]
    ReadOnly,
    #[error("invalid text range: start={start}, end={end}, len={len}")]
    InvalidRange {
        start: usize,
        end: usize,
        len: usize,
    },
    #[error("conflicting shortcut binding: {binding}")]
    ConflictingShortcut { binding: String },
    #[error("clipboard error: {0}")]
    Clipboard(String),
    #[error("editor error: {0}")]
    Internal(String),
}
