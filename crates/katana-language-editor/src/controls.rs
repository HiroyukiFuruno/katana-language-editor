use crate::{
    EditorDocumentState, EditorDocumentUpdate, EditorDocumentUpdateReport,
    EditorExternalUndoRecord, EditorResult, Selection, TextContent, TextOffset, TextRange,
};

pub trait EditorWriteAccess {
    fn insert_at(&mut self, offset: TextOffset, text: &str) -> EditorResult<()>;
    fn replace_range(&mut self, range: TextRange, text: &str) -> EditorResult<()>;
    fn set_text(&mut self, text: String) -> EditorResult<()>;
    fn apply_batch(&mut self, edits: Vec<(TextRange, String)>) -> EditorResult<()>;
    fn set_write_origin(&mut self, origin: Option<String>);

    fn with_origin(&mut self, tag: impl Into<String>) -> WriteHandle<'_, Self>
    where
        Self: Sized,
    {
        WriteHandle::new(self, tag.into())
    }
}

pub struct WriteHandle<'a, E: EditorWriteAccess + ?Sized> {
    editor: &'a mut E,
    origin: String,
}

impl<'a, E: EditorWriteAccess + ?Sized> WriteHandle<'a, E> {
    pub fn new(editor: &'a mut E, origin: String) -> Self {
        Self { editor, origin }
    }

    pub fn insert_at(&mut self, offset: TextOffset, text: &str) -> EditorResult<()> {
        self.run(|editor| editor.insert_at(offset, text))
    }

    pub fn replace_range(&mut self, range: TextRange, text: &str) -> EditorResult<()> {
        self.run(|editor| editor.replace_range(range, text))
    }

    pub fn set_text(&mut self, text: String) -> EditorResult<()> {
        self.run(|editor| editor.set_text(text))
    }

    pub fn apply_batch(&mut self, edits: Vec<(TextRange, String)>) -> EditorResult<()> {
        self.run(|editor| editor.apply_batch(edits))
    }

    fn run(&mut self, action: impl FnOnce(&mut E) -> EditorResult<()>) -> EditorResult<()> {
        self.editor.set_write_origin(Some(self.origin.clone()));
        let result = action(self.editor);
        self.editor.set_write_origin(None);
        result
    }
}

pub trait EditorHistoryControl {
    fn undo(&mut self) -> EditorResult<()>;
    fn redo(&mut self) -> EditorResult<()>;
    fn can_undo(&self) -> bool;
    fn can_redo(&self) -> bool;
    fn push_history_barrier(&mut self) -> EditorResult<()>;
}

pub trait EditorDocumentStateControl {
    fn document_state(&self) -> EditorDocumentState;
    fn set_document_state(&mut self, state: EditorDocumentState) -> EditorResult<()>;
    fn apply_document_update(
        &mut self,
        update: EditorDocumentUpdate,
    ) -> EditorResult<EditorDocumentUpdateReport>;
    fn mark_saved(&mut self) -> EditorResult<EditorDocumentUpdateReport>;
    fn pending_external_undo(&self) -> Option<EditorExternalUndoRecord>;
}

pub trait EditorSelectionControl {
    fn set_cursor(&mut self, offset: TextOffset) -> EditorResult<()>;
    fn set_selection(&mut self, selection: Selection) -> EditorResult<()>;
    fn selections(&self) -> Vec<Selection>;
}

pub trait EditorCursorRestoreControl {
    fn queue_cursor_restore(&mut self, range: TextRange) -> EditorResult<()>;
    fn apply_pending_cursor_restore(&mut self) -> EditorResult<Option<Selection>>;
}

pub trait EditorClipboardControl {
    fn cut(&mut self) -> EditorResult<TextContent>;
    fn copy(&mut self) -> EditorResult<TextContent>;
    fn paste(&mut self) -> EditorResult<()>;
}
