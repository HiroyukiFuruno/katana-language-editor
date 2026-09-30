use super::*;

#[test]
fn identity_is_scoped_by_workspace_and_document() {
    let first = EditorDocumentIdentity::with_workspace("workspace-a", "doc.md");
    let second = EditorDocumentIdentity::with_workspace("workspace-b", "doc.md");

    assert_ne!(first.stable_id_source(), second.stable_id_source());
}

#[test]
fn identity_source_encodes_component_boundaries_unambiguously() {
    let first = EditorDocumentIdentity::with_workspace("workspace::draft", "readme.md");
    let second = EditorDocumentIdentity::with_workspace("workspace", "draft::readme.md");
    let unscoped = EditorDocumentIdentity::new("workspace::draft::readme.md");

    assert_ne!(first.stable_id_source(), second.stable_id_source());
    assert_ne!(first.stable_id_source(), unscoped.stable_id_source());
    assert_ne!(second.stable_id_source(), unscoped.stable_id_source());
}

#[test]
fn identity_source_uses_utf8_byte_lengths() {
    let identity = EditorDocumentIdentity::with_workspace("作業場", "⭐\u{fe0f}.md");

    assert_eq!(
        identity.stable_id_source(),
        "workspace:9:作業場;document:9:⭐\u{fe0f}.md"
    );
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
