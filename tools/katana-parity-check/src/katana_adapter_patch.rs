use crate::actual_repo_adapter;
use crate::matrix;

pub(crate) const PATCH_DOCUMENT: &str =
    include_str!("../../../docs/v0-1-0-katana-downstream-adapter-patch.md");

pub(crate) const REQUIRED_PATCH_EVIDENCE: &[&str] = &[
    "crates/katana-ui/Cargo.toml",
    "crates/katana-ui/tests/ui_integration_parallel.rs",
    "crates/katana-ui/tests/integration/editor/kle_downstream_adapter.rs",
    "katana-language-editor = { path = \"../../../katana-language-editor/crates/katana-language-editor\" }",
    "katana-language-editor-egui = { path = \"../../../katana-language-editor/crates/katana-language-editor-egui\" }",
    "cargo test -p katana-ui --test ui_integration_parallel editor_kle_downstream_adapter -- --nocapture",
    "just katana-repo-adapter-check",
    "kle_event_action_stream_updates_real_katana_editor_state",
    "KatanaKleEditorAdapter::drain_editor",
    "EguiLanguageEditor::new(editor_config())",
    "EditorAction::SaveDocument",
    "Scratch Validation",
    "It does not close OpenSpec `12.4b`",
    "active_document_buffer(&harness)",
    "KatanA document should become dirty after KLE content event",
    "EditorScrollSource::Preview",
    ".with_target_line(7)",
    "ScrollSource::Neither",
    "last_scroll_to_line",
    "std::fs::read_to_string(&second_path)",
];

pub(crate) const FORBIDDEN_PATCH_EVIDENCE: &[&str] = &[
    "force_view_mode",
    "# Updated from KLE adapter",
    "katana_language_editor::LanguageEditor::poll_events",
    "Arc<dyn SyntaxHighlighter>",
];
pub(crate) const REQUIRED_ADAPTER_SEQUENCE: &[&str] = &[
    ".set_text(\"beta\\nupdated by KLE\".to_string())",
    "KatanaKleEditorAdapter::drain_editor(&mut editor, &mut harness);",
    "active_document_buffer(&harness)",
    "KatanA document should become dirty after KLE content event",
    "EditorAction::SetViewMode(EditorViewMode::Split)",
    "EditorAction::SetSplitDirection(EditorSplitDirection::Vertical)",
    "EditorScrollSource::Preview",
    ".with_target_line(7)",
    "EditorAction::NavigateDocumentSearch",
    "EditorAction::SelectAndJump(EditorSelectAndJumpRequest::new(1))",
    "EditorAction::ToggleProblemsPanel",
    "EditorAction::SetProblemsScope(EditorProblemsScope::ActiveDocument)",
    "KatanaKleEditorAdapter::drain_editor(&mut editor, &mut harness);",
    "assert_eq!(state.active_view_mode(), ViewMode::Split);",
    "assert_eq!(state.scroll.source, ScrollSource::Neither);",
    "assert_eq!(state.scroll.last_scroll_to_line, Some(1));",
    "request(&mut editor, EditorAction::SaveDocument);",
    "std::fs::read_to_string(&second_path)",
    "KatanA save action should mark the document clean",
    "std::fs::read_to_string(&first_path)",
];

pub(crate) struct KatanaAdapterPatchAudit;

impl KatanaAdapterPatchAudit {
    pub(crate) fn validate() -> Result<(), String> {
        for needle in REQUIRED_PATCH_EVIDENCE {
            if !PATCH_DOCUMENT.contains(needle) {
                return Err(format!("KatanA adapter patch draft is missing: {needle}"));
            }
        }
        for needle in FORBIDDEN_PATCH_EVIDENCE {
            if PATCH_DOCUMENT.contains(needle) {
                return Err(format!(
                    "KatanA adapter patch draft contains stale evidence: {needle}"
                ));
            }
        }
        let has_open_gaps = matrix::FEATURES
            .iter()
            .any(|feature| feature.has_open_gap());
        actual_repo_adapter::ActualRepoAdapterEvidence::validate_task_state_matches_actual_repo(
            tasks_document(),
            has_open_gaps,
        )?;
        validate_adapter_sequence()?;
        Ok(())
    }
}

fn validate_adapter_sequence() -> Result<(), String> {
    let mut offset = 0;
    for needle in REQUIRED_ADAPTER_SEQUENCE {
        let remaining = &PATCH_DOCUMENT[offset..];
        let Some(position) = remaining.find(needle) else {
            return Err(format!(
                "KatanA adapter patch draft is missing ordered evidence: {needle}"
            ));
        };
        offset += position + needle.len();
    }
    Ok(())
}

fn tasks_document() -> &'static str {
    include_str!("../../../openspec/changes/v0-1-0-language-editor-extraction/tasks.md")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patch_draft_records_required_katana_adapter_evidence() -> Result<(), String> {
        KatanaAdapterPatchAudit::validate()
    }

    #[test]
    fn actual_katana_adapter_task_state_matches_repo_evidence() -> Result<(), String> {
        let has_open_gaps = matrix::FEATURES
            .iter()
            .any(|feature| feature.has_open_gap());
        actual_repo_adapter::ActualRepoAdapterEvidence::validate_task_state_matches_actual_repo(
            tasks_document(),
            has_open_gaps,
        )
    }

    #[test]
    fn adapter_patch_preserves_scratch_validated_event_order() -> Result<(), String> {
        validate_adapter_sequence()
    }
}
