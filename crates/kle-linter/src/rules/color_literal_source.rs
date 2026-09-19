use crate::diagnostics::KleLintError;
use std::path::{Path, PathBuf};

pub(super) struct ColorLiteralSource<'source> {
    file: PathBuf,
    source: &'source str,
}

impl<'source> ColorLiteralSource<'source> {
    pub(super) fn new(file: PathBuf, source: &'source str) -> Self {
        Self { file, source }
    }

    pub(super) fn file(&self) -> &Path {
        &self.file
    }

    pub(super) fn capture(&self, span: proc_macro2::Span) -> Result<String, KleLintError> {
        let start_location = span.start();
        let end_location = span.end();
        let start = self.offset(start_location.line, start_location.column)?;
        let end = self.offset(end_location.line, end_location.column)?;
        if start >= end {
            return Err(self.error(start_location.line, start_location.column + 1));
        }
        self.source
            .get(start..end)
            .map(ToOwned::to_owned)
            .ok_or_else(|| self.error(start_location.line, start_location.column + 1))
    }

    fn offset(&self, line: usize, column: usize) -> Result<usize, KleLintError> {
        if line == 0 {
            return Err(self.error(line, column + 1));
        }
        let mut offset = 0;
        for _ in 1..line {
            let Some(newline) = self.source[offset..].find('\n') else {
                return Err(self.error(line, column + 1));
            };
            offset += newline + 1;
        }
        let line_source = self.line_source(offset);
        let byte = line_source
            .char_indices()
            .nth(column)
            .map_or_else(
                || (column == line_source.chars().count()).then_some(line_source.len()),
                |(byte, _)| Some(byte),
            )
            .ok_or_else(|| self.error(line, column + 1))?;
        Ok(offset + byte)
    }

    fn line_source(&self, offset: usize) -> &str {
        let remaining = &self.source[offset..];
        let line = match remaining.find('\n') {
            Some(end) => &remaining[..end],
            None => remaining,
        };
        match line.strip_suffix('\r') {
            Some(line) => line,
            None => line,
        }
    }

    fn error(&self, line: usize, column: usize) -> KleLintError {
        KleLintError::DiagnosticSpan {
            path: self.file.clone(),
            line,
            column,
        }
    }
}

#[cfg(test)]
#[path = "color_literal_source_tests.rs"]
mod color_literal_source_tests;
