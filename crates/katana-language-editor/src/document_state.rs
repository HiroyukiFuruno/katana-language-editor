use crate::content::TextContent;
use serde::{Deserialize, Serialize};

const ID_SEPARATOR: &str = "::";
const ORIGIN_USER_INPUT: &str = "user-input";
const ORIGIN_HOST_EXTERNAL_CHANGE: &str = "host-external-change";
const ORIGIN_DISK_REFRESH: &str = "disk-refresh";

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorDocumentIdentity {
    pub workspace_id: Option<String>,
    pub document_id: String,
}

impl EditorDocumentIdentity {
    pub fn new(document_id: impl Into<String>) -> Self {
        Self {
            workspace_id: None,
            document_id: document_id.into(),
        }
    }

    pub fn with_workspace(workspace_id: impl Into<String>, document_id: impl Into<String>) -> Self {
        Self {
            workspace_id: Some(workspace_id.into()),
            document_id: document_id.into(),
        }
    }

    pub fn stable_id_source(&self) -> String {
        match &self.workspace_id {
            Some(workspace_id) => {
                format!("{workspace_id}{ID_SEPARATOR}{}", self.document_id)
            }
            None => self.document_id.clone(),
        }
    }
}

impl Default for EditorDocumentIdentity {
    fn default() -> Self {
        Self::new("untitled")
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorDocumentState {
    pub identity: EditorDocumentIdentity,
    pub dirty: bool,
    pub loaded: bool,
    pub read_only: bool,
    pub reference: bool,
    pub virtual_document: bool,
}

impl EditorDocumentState {
    pub fn new(identity: EditorDocumentIdentity) -> Self {
        Self {
            identity,
            dirty: false,
            loaded: true,
            read_only: false,
            reference: false,
            virtual_document: false,
        }
    }

    pub const fn is_effectively_read_only(&self) -> bool {
        self.read_only || self.reference
    }
}

impl Default for EditorDocumentState {
    fn default() -> Self {
        Self::new(EditorDocumentIdentity::default())
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorDocumentUpdateOrigin {
    UserInput,
    HostExternalChange,
    DiskRefresh,
}

impl EditorDocumentUpdateOrigin {
    pub const fn tag(self) -> &'static str {
        match self {
            Self::UserInput => ORIGIN_USER_INPUT,
            Self::HostExternalChange => ORIGIN_HOST_EXTERNAL_CHANGE,
            Self::DiskRefresh => ORIGIN_DISK_REFRESH,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorDocumentUpdate {
    pub content: TextContent,
    pub origin: EditorDocumentUpdateOrigin,
    pub mark_dirty: bool,
    pub record_external_undo: bool,
    pub refresh_preview: bool,
    pub refresh_search: bool,
    pub refresh_diagnostics: bool,
}

impl EditorDocumentUpdate {
    pub fn new(content: TextContent, origin: EditorDocumentUpdateOrigin) -> Self {
        Self {
            content,
            origin,
            mark_dirty: true,
            record_external_undo: false,
            refresh_preview: true,
            refresh_search: true,
            refresh_diagnostics: true,
        }
    }

    pub fn host_external_change(content: TextContent) -> Self {
        Self {
            record_external_undo: true,
            ..Self::new(content, EditorDocumentUpdateOrigin::HostExternalChange)
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorDocumentUpdateReport {
    pub content_changed: bool,
    pub dirty: bool,
    pub external_undo_pending: bool,
    pub preview_refresh_required: bool,
    pub search_refresh_required: bool,
    pub diagnostics_refresh_required: bool,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorExternalUndoRecord {
    pub identity: EditorDocumentIdentity,
    pub before: TextContent,
    pub after: TextContent,
}

impl EditorExternalUndoRecord {
    pub fn new(identity: EditorDocumentIdentity, before: TextContent, after: TextContent) -> Self {
        Self {
            identity,
            before,
            after,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_is_scoped_by_workspace_and_document() {
        let first = EditorDocumentIdentity::with_workspace("workspace-a", "doc.md");
        let second = EditorDocumentIdentity::with_workspace("workspace-b", "doc.md");

        assert_ne!(first.stable_id_source(), second.stable_id_source());
    }

    #[test]
    fn reference_document_is_effectively_read_only() {
        let mut state = EditorDocumentState::new(EditorDocumentIdentity::new("doc.md"));
        state.reference = true;

        assert!(state.is_effectively_read_only());
    }

    #[test]
    fn host_external_change_requests_undo_and_refreshes() {
        let update = EditorDocumentUpdate::host_external_change(TextContent::new("after"));

        assert!(update.record_external_undo);
        assert!(update.refresh_preview);
        assert!(update.refresh_search);
        assert!(update.refresh_diagnostics);
        assert_eq!(update.origin.tag(), ORIGIN_HOST_EXTERNAL_CHANGE);
    }
}
