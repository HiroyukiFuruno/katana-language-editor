use crate::{
    CursorPosition, EditorClipboardPasteReport, EditorClipboardPasteRequest, Selection, TextContent,
};
use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorEvent {
    ContentChanged(TextContent),
    ContentChangedWithOrigin {
        content: TextContent,
        origin: String,
    },
    CursorMoved(CursorPosition),
    SelectionChanged(Selection),
    SaveRequested,
    AutosaveRequested,
    ClipboardPasteRequested(EditorClipboardPasteRequest),
    ClipboardPasteResolved(EditorClipboardPasteReport),
}

#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorOutput {
    pub cursor: CursorPosition,
    pub selection: Option<Selection>,
}
