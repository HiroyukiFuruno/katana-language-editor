use crate::document_state::EditorDocumentUpdateReport;

impl EditorDocumentUpdateReport {
    pub const fn changed(
        dirty: bool,
        external_undo_pending: bool,
        preview_refresh_required: bool,
        search_refresh_required: bool,
        diagnostics_refresh_required: bool,
    ) -> Self {
        Self {
            content_changed: true,
            dirty,
            external_undo_pending,
            preview_refresh_required,
            search_refresh_required,
            diagnostics_refresh_required,
        }
    }

    pub const fn unchanged(dirty: bool) -> Self {
        Self {
            content_changed: false,
            dirty,
            external_undo_pending: false,
            preview_refresh_required: false,
            search_refresh_required: false,
            diagnostics_refresh_required: false,
        }
    }

    pub const fn saved() -> Self {
        Self {
            content_changed: false,
            dirty: false,
            external_undo_pending: false,
            preview_refresh_required: false,
            search_refresh_required: false,
            diagnostics_refresh_required: true,
        }
    }
}
