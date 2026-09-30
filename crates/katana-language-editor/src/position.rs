use crate::{EditorError, EditorResult};
use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextOffset {
    pub char_index: usize,
}

impl TextOffset {
    pub const fn new(char_index: usize) -> Self {
        Self { char_index }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextRange {
    pub start: usize,
    pub end: usize,
}

impl TextRange {
    pub fn new(start: usize, end: usize) -> EditorResult<Self> {
        if start <= end {
            return Ok(Self { start, end });
        }
        Err(EditorError::InvalidRange {
            start,
            end,
            len: end,
        })
    }

    pub fn validate_for_len(self, len: usize) -> EditorResult<Self> {
        if self.start <= self.end && self.end <= len {
            return Ok(self);
        }
        Err(EditorError::InvalidRange {
            start: self.start,
            end: self.end,
            len,
        })
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CursorPosition {
    pub line: usize,
    pub column: usize,
    pub offset: TextOffset,
}

impl CursorPosition {
    pub const fn new(line: usize, column: usize, char_index: usize) -> Self {
        Self {
            line,
            column,
            offset: TextOffset::new(char_index),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selection {
    pub start: CursorPosition,
    pub end: CursorPosition,
}

impl Selection {
    pub const fn new(start: CursorPosition, end: CursorPosition) -> Self {
        Self { start, end }
    }

    pub const fn collapsed(cursor: CursorPosition) -> Self {
        Self {
            start: cursor,
            end: cursor,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisibleRange {
    pub start_line: usize,
    pub end_line: usize,
}

impl VisibleRange {
    pub const fn new(start_line: usize, end_line: usize) -> Self {
        Self {
            start_line,
            end_line,
        }
    }
}
