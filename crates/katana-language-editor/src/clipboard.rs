use crate::{EditorDocumentIdentity, EditorResult, TextContent, TextRange};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_PASTE_TOKEN: AtomicU64 = AtomicU64::new(1);

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClipboardPasteToken(u64);

impl ClipboardPasteToken {
    pub fn next() -> Self {
        Self(NEXT_PASTE_TOKEN.fetch_add(1, Ordering::Relaxed))
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorClipboardPasteRequest {
    pub token: ClipboardPasteToken,
    pub document_identity: EditorDocumentIdentity,
    pub range: TextRange,
    pub content: TextContent,
}

impl EditorClipboardPasteRequest {
    pub fn new(
        token: ClipboardPasteToken,
        document_identity: EditorDocumentIdentity,
        range: TextRange,
        content: TextContent,
    ) -> Self {
        Self {
            token,
            document_identity,
            range,
            content,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorClipboardPasteResolution {
    Text(TextContent),
    Image,
    NoPayload,
    Failed,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorClipboardPasteRejection {
    NoPendingRequest,
    TokenMismatch,
    RequestMismatch,
    DocumentMismatch,
    ContentMismatch,
    ReadOnly,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorClipboardPasteStatus {
    TextApplied,
    ImageRequested,
    NoPayloadConsumed,
    FailureConsumed,
    Rejected(EditorClipboardPasteRejection),
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorClipboardPasteReport {
    pub status: EditorClipboardPasteStatus,
}

impl EditorClipboardPasteReport {
    pub const fn accepted(status: EditorClipboardPasteStatus) -> Self {
        Self { status }
    }
}

pub trait ClipboardBackend: Send + Sync {
    fn read_text(&self) -> EditorResult<String>;
    fn write_text(&self, text: &str) -> EditorResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_monotonic() {
        let first = ClipboardPasteToken::next();
        let second = ClipboardPasteToken::next();
        assert!(second.0 > first.0);
    }

    #[test]
    fn request_carries_char_range_and_snapshot() -> EditorResult<()> {
        let range = TextRange::new(1, 3)?;
        let request = EditorClipboardPasteRequest::new(
            ClipboardPasteToken::next(),
            EditorDocumentIdentity::new("doc"),
            range,
            TextContent::new("日本語⭐️"),
        );
        assert_eq!(request.range, range);
        assert_eq!(request.content.text, "日本語⭐️");
        Ok(())
    }
}
