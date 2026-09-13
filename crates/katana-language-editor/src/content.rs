use crate::{EditorError, EditorResult, TextRange};
use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextContent {
    pub text: String,
    pub language: Option<String>,
}

impl TextContent {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            language: None,
        }
    }

    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    pub fn char_count(&self) -> usize {
        self.text.chars().count()
    }

    pub fn byte_index_for_char(&self, char_index: usize) -> Option<usize> {
        if char_index == self.char_count() {
            return Some(self.text.len());
        }
        self.text.char_indices().nth(char_index).map(|(idx, _)| idx)
    }

    pub fn slice_char_range(&self, range: TextRange) -> Option<&str> {
        let start = self.byte_index_for_char(range.start)?;
        let end = self.byte_index_for_char(range.end)?;
        self.text.get(start..end)
    }

    pub fn replace_char_range(&mut self, range: TextRange, replacement: &str) -> EditorResult<()> {
        let valid = range.validate_for_len(self.char_count())?;
        let start = self.byte_index_for_char(valid.start);
        let end = self.byte_index_for_char(valid.end);
        let (Some(start), Some(end)) = (start, end) else {
            return Err(EditorError::InvalidRange {
                start: valid.start,
                end: valid.end,
                len: self.char_count(),
            });
        };
        self.text.replace_range(start..end, replacement);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_grapheme_text_by_character_range() -> EditorResult<()> {
        let mut content = TextContent::new("a🙂b");
        content.replace_char_range(TextRange::new(1, 2)?, "x")?;
        assert_eq!("axb", content.text);
        Ok(())
    }
}
