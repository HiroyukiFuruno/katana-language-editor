use crate::{
    EditorClipboardPasteReport, EditorClipboardPasteRequest, EditorClipboardPasteResolution,
    EditorResult,
};

/// Resolves a host-controlled clipboard paste request for the active editor document.
pub trait EditorClipboardPasteControl {
    fn resolve_clipboard_paste(
        &mut self,
        request: EditorClipboardPasteRequest,
        resolution: EditorClipboardPasteResolution,
    ) -> EditorResult<EditorClipboardPasteReport>;
}
