# Rejected Historical KatanA Adapter Patch (Do Not Apply)

## Status

This document records a rejected approach and is not an implementation plan.
KatanA is a read-only reference for KLE v0.1.0: no KatanA source, manifest,
test registration, branch, worktree, or patch may be created or changed for
this goal. **No command or code block below may be applied.**

The valid downstream evidence route is the KLE-owned `tools/katana-host-e2e`
test target, which consumes the fixed KatanA checkout as a read-only path
dependency. The parity checker must reject a planned or absent KatanA adapter
as evidence. This historical content is retained only to make that rejected
route auditable.

## Rejected scratch record

## Scratch Validation

- Scratch clone: `/Users/hiroyuki_furuno/works/private/katana-language-editor/tmp/katana-kle-adapter-verify`
- Command: `cargo test -p katana-ui --test ui_integration_parallel editor_kle_downstream_adapter -- --nocapture`
- Result: PASS on 2026-07-03, 1 passed / 139 filtered out.
- Scope: this proves the test-only adapter patch compiles and runs against a local KatanA clone. It does not close OpenSpec `12.4b` until the patch is applied to the actual KatanA repository or the user explicitly accepts scratch-clone evidence as equivalent.

Scratch validation found and fixed these parity drifts:

- `SelectAndJump` must preserve Split / CodeOnly and only apply the CodeOnly fallback when the current KatanA view mode is PreviewOnly.
- Programmatic target-line scroll must suppress scroll source and leave KatanA `ScrollSource::Neither`.
- Post-frame KatanA state consumes `scroll_to_line`; validation checks `last_scroll_to_line` for the consumed target.

## `crates/katana-ui/Cargo.toml`

Add these dev-dependencies:

```toml
[dev-dependencies]
accesskit = "0.24.1"
egui_kittest = { version = "0.35", features = ["snapshot", "wgpu", "eframe"] }
katana-language-editor = { path = "../../../katana-language-editor/crates/katana-language-editor" }
katana-language-editor-egui = { path = "../../../katana-language-editor/crates/katana-language-editor-egui" }
time = "0.3"
```

## `crates/katana-ui/tests/ui_integration_parallel.rs`

Wire the test module next to `editor_ui`:

```rust
#[path = "integration/editor/ui.rs"]
mod editor_ui;
#[path = "integration/editor/kle_downstream_adapter.rs"]
mod editor_kle_downstream_adapter;
```

## `crates/katana-ui/tests/integration/editor/kle_downstream_adapter.rs`

```rust
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use katana_language_editor::{
    AutosavePolicy, ClipboardBackend, ColorTokens, ColorTokensInput, EditorAction,
    EditorActionControl, EditorActionRequest, EditorActionSource, EditorConfig, EditorConfigInput,
    EditorDocumentSearchNavigation, EditorEvent, EditorLogicalScrollPosition, EditorProblemsScope,
    EditorResult, EditorScrollSource, EditorScrollSyncRequest, EditorSelectAndJumpRequest,
    EditorSplitDirection, EditorViewMode, EditorWriteAccess, LanguageEditor, Locale,
    NoopSyntaxHighlighter, Rgba, SearchDirection, ShortcutMap, Spacing, Strings, StringsInput,
    TextDirection, Theme, Typography,
};
use katana_language_editor_egui::EguiLanguageEditor;
use katana_ui::app_state::{AppAction, ProblemsScope, ScrollSource, ViewMode};

use crate::integration::harness_utils::{fresh_temp_dir, setup_harness, wait_for_workspace_load};

struct MemoryClipboard {
    text: Mutex<String>,
}

impl MemoryClipboard {
    fn new() -> Self {
        Self {
            text: Mutex::new(String::new()),
        }
    }
}

impl ClipboardBackend for MemoryClipboard {
    fn read_text(&self) -> EditorResult<String> {
        Ok(self
            .text
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone())
    }

    fn write_text(&self, text: &str) -> EditorResult<()> {
        *self
            .text
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = text.to_string();
        Ok(())
    }
}

struct KatanaKleEditorAdapter;

impl KatanaKleEditorAdapter {
    fn drain_editor(
        editor: &mut EguiLanguageEditor,
        harness: &mut egui_kittest::Harness<'static, katana_ui::shell::KatanaApp>,
    ) {
        for event in editor.poll_events() {
            Self::apply_event(event, harness);
        }
    }

    fn apply_event(
        event: EditorEvent,
        harness: &mut egui_kittest::Harness<'static, katana_ui::shell::KatanaApp>,
    ) {
        match event {
            EditorEvent::ContentChanged(content)
            | EditorEvent::ContentChangedWithOrigin { content, .. } => {
                Self::dispatch(harness, AppAction::UpdateBuffer(content.text));
            }
            EditorEvent::ActionRequested(request) => Self::apply_request(request, harness),
            EditorEvent::SaveRequested | EditorEvent::AutosaveRequested => {
                Self::dispatch(harness, AppAction::SaveDocument);
            }
            EditorEvent::CursorMoved(_)
            | EditorEvent::SelectionChanged(_)
            | EditorEvent::DiagnosticsChanged(_) => {}
            _ => {}
        }
    }

    fn apply_request(
        request: EditorActionRequest,
        harness: &mut egui_kittest::Harness<'static, katana_ui::shell::KatanaApp>,
    ) {
        match request.action {
            EditorAction::SaveDocument => Self::dispatch(harness, AppAction::SaveDocument),
            EditorAction::SetViewMode(mode) => {
                Self::dispatch(harness, AppAction::SetViewMode(Self::view_mode(mode)));
            }
            EditorAction::ToggleSplitMode => Self::dispatch(harness, AppAction::ToggleSplitMode),
            EditorAction::ToggleCodePreview => {
                Self::dispatch(harness, AppAction::ToggleCodePreview);
            }
            EditorAction::SetSplitDirection(direction) => {
                Self::dispatch(
                    harness,
                    AppAction::SetSplitDirection(Self::split_direction(direction)),
                );
            }
            EditorAction::SyncScroll(request) => Self::apply_scroll_sync(request, harness),
            EditorAction::SelectAndJump(request) => {
                Self::apply_select_and_jump(request, harness);
            }
            EditorAction::NavigateDocumentSearch(request) => {
                Self::apply_document_search_navigation(request, harness);
            }
            EditorAction::ToggleProblemsPanel => {
                Self::dispatch(harness, AppAction::ToggleProblemsPanel);
            }
            EditorAction::SetProblemsScope(scope) => {
                harness.state_mut().app_state_mut().diagnostics.scope =
                    Self::problems_scope(scope);
            }
            EditorAction::IngestImageFile => Self::dispatch(harness, AppAction::IngestImageFile),
            EditorAction::IngestClipboardImage
            | EditorAction::IngestClipboardFileUrls { .. } => {
                Self::dispatch(harness, AppAction::IngestClipboardImage);
            }
            EditorAction::FormatDocument
            | EditorAction::ApplyDiagnosticFix { .. }
            | EditorAction::ApplyAllDiagnosticFixes { .. }
            | EditorAction::OpenDiagnosticDocumentation { .. }
            | EditorAction::ApplyProblemsFixes { .. }
            | EditorAction::ActivateGutterLine { .. }
            | EditorAction::RunAuthoringCommand { .. }
            | EditorAction::CloseTransientUi
            | EditorAction::HostCommand { .. } => {}
            _ => {}
        }
    }

    fn dispatch(
        harness: &mut egui_kittest::Harness<'static, katana_ui::shell::KatanaApp>,
        action: AppAction,
    ) {
        harness.state_mut().trigger_action(action);
        harness.run_steps(3);
    }

    fn apply_select_and_jump(
        request: EditorSelectAndJumpRequest,
        harness: &mut egui_kittest::Harness<'static, katana_ui::shell::KatanaApp>,
    ) {
        let path = harness
            .state()
            .app_state_for_test()
            .active_path()
            .expect("active document must exist before KLE select-and-jump");
        Self::dispatch(
            harness,
            AppAction::SelectDocumentAndJump {
                path,
                line: request.line_index,
                byte_range: 0..0,
            },
        );
    }

    fn apply_scroll_sync(
        request: EditorScrollSyncRequest,
        harness: &mut egui_kittest::Harness<'static, katana_ui::shell::KatanaApp>,
    ) {
        let state = harness.state_mut().app_state_mut();
        state.scroll.source = if request.suppress_source_update {
            ScrollSource::Neither
        } else {
            match request.source {
                EditorScrollSource::Editor => ScrollSource::Editor,
                EditorScrollSource::Preview => ScrollSource::Preview,
                EditorScrollSource::Neither | EditorScrollSource::Host => ScrollSource::Neither,
                _ => ScrollSource::Neither,
            }
        };
        state.scroll.logical_position.segment_index = request.logical_position.segment_index;
        state.scroll.logical_position.progress =
            f32::from(request.logical_position.progress_per_mille) / 1000.0;
        if let Some(line) = request.target_line {
            state.scroll.last_scroll_to_line = None;
            state.scroll.scroll_to_line = Some(line);
        }
    }

    fn apply_document_search_navigation(
        request: EditorDocumentSearchNavigation,
        harness: &mut egui_kittest::Harness<'static, katana_ui::shell::KatanaApp>,
    ) {
        let state = harness.state_mut().app_state_mut();
        if request.reset_last_scroll_target {
            state.scroll.last_scroll_to_line = None;
        }
        if let Some(line) = request.target_line {
            state.scroll.scroll_to_line = Some(line);
        }
        state.search.doc_search_active_index = match request.direction {
            SearchDirection::Next => state.search.doc_search_active_index.saturating_add(1),
            SearchDirection::Previous => state.search.doc_search_active_index.saturating_sub(1),
            _ => state.search.doc_search_active_index,
        };
    }

    fn view_mode(mode: EditorViewMode) -> ViewMode {
        match mode {
            EditorViewMode::PreviewOnly => ViewMode::PreviewOnly,
            EditorViewMode::Split => ViewMode::Split,
            EditorViewMode::CodeOnly => ViewMode::CodeOnly,
            _ => ViewMode::CodeOnly,
        }
    }

    fn split_direction(direction: EditorSplitDirection) -> katana_platform::SplitDirection {
        match direction {
            EditorSplitDirection::Horizontal => katana_platform::SplitDirection::Horizontal,
            EditorSplitDirection::Vertical => katana_platform::SplitDirection::Vertical,
            _ => katana_platform::SplitDirection::Horizontal,
        }
    }

    fn problems_scope(scope: EditorProblemsScope) -> ProblemsScope {
        match scope {
            EditorProblemsScope::OpenTabs => ProblemsScope::OpenTabs,
            EditorProblemsScope::ActiveDocument => ProblemsScope::ActiveTab,
            _ => ProblemsScope::OpenTabs,
        }
    }
}

#[test]
fn kle_event_action_stream_updates_real_katana_editor_state() {
    let mut harness = setup_harness();
    harness.step();
    let first_path = open_document(&mut harness, "katana_test_kle_adapter", "first.md", "alpha");
    let second_path = open_second_document(&mut harness, &first_path);
    let mut editor = EguiLanguageEditor::new(editor_config());

    editor
        .set_text("beta\nupdated by KLE".to_string())
        .expect("KLE text update must emit a content event");
    KatanaKleEditorAdapter::drain_editor(&mut editor, &mut harness);
    assert_eq!(
        active_document_buffer(&harness),
        "beta\nupdated by KLE",
        "KLE content event must update the active KatanA buffer"
    );
    assert!(
        harness
            .state()
            .app_state_for_test()
            .active_document()
            .expect("active document should exist")
            .is_dirty,
        "KatanA document should become dirty after KLE content event"
    );

    request(&mut editor, EditorAction::SetViewMode(EditorViewMode::Split));
    request(
        &mut editor,
        EditorAction::SetSplitDirection(EditorSplitDirection::Vertical),
    );
    request(
        &mut editor,
        EditorAction::SyncScroll(
            EditorScrollSyncRequest::new(
                EditorScrollSource::Preview,
                EditorLogicalScrollPosition::new(4, 625),
            )
            .with_target_line(7),
        ),
    );
    request(
        &mut editor,
        EditorAction::NavigateDocumentSearch(
            EditorDocumentSearchNavigation::new(SearchDirection::Next).with_target_line(2),
        ),
    );
    request(
        &mut editor,
        EditorAction::SelectAndJump(EditorSelectAndJumpRequest::new(1)),
    );
    request(&mut editor, EditorAction::ToggleProblemsPanel);
    request(
        &mut editor,
        EditorAction::SetProblemsScope(EditorProblemsScope::ActiveDocument),
    );
    KatanaKleEditorAdapter::drain_editor(&mut editor, &mut harness);

    let state = harness.state().app_state_for_test();
    assert_eq!(state.active_path(), Some(second_path.clone()));
    assert_eq!(state.active_view_mode(), ViewMode::Split);
    assert_eq!(
        state.active_split_direction(),
        katana_platform::SplitDirection::Vertical
    );
    assert_eq!(state.scroll.source, ScrollSource::Neither);
    assert_eq!(state.scroll.logical_position.segment_index, 4);
    assert_eq!(state.scroll.logical_position.progress, 0.625);
    assert_eq!(state.scroll.scroll_to_line, None);
    assert_eq!(state.scroll.last_scroll_to_line, Some(1));
    assert!(state.diagnostics.is_panel_open);
    assert_eq!(state.diagnostics.scope, ProblemsScope::ActiveTab);
    assert_eq!(state.search.doc_search_active_index, 1);

    request(&mut editor, EditorAction::SaveDocument);
    KatanaKleEditorAdapter::drain_editor(&mut editor, &mut harness);
    assert_eq!(
        std::fs::read_to_string(&second_path).expect("saved document should be readable"),
        "beta\nupdated by KLE"
    );
    assert!(
        !harness
            .state()
            .app_state_for_test()
            .active_document()
            .expect("active document should exist")
            .is_dirty,
        "KatanA save action should mark the document clean"
    );
    assert_eq!(
        std::fs::read_to_string(&first_path).expect("first document should be readable"),
        "alpha",
        "KLE update must stay scoped to the active KatanA document"
    );
}

fn request(editor: &mut EguiLanguageEditor, action: EditorAction) {
    editor
        .request_action(EditorActionRequest::new(action, EditorActionSource::Host))
        .expect("KLE action request must be queued");
}

fn open_document(
    harness: &mut egui_kittest::Harness<'static, katana_ui::shell::KatanaApp>,
    prefix: &str,
    filename: &str,
    content: &str,
) -> PathBuf {
    let temp_dir = fresh_temp_dir(prefix);
    let file_path = temp_dir.join(filename);
    std::fs::write(&file_path, content).expect("fixture must be written");
    let file_path = file_path
        .canonicalize()
        .expect("fixture path must be canonicalized");

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir));
    wait_for_workspace_load(harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(file_path.clone()));
    harness.run_steps(5);
    file_path
}

fn open_second_document(
    harness: &mut egui_kittest::Harness<'static, katana_ui::shell::KatanaApp>,
    first_path: &PathBuf,
) -> PathBuf {
    let second_path = first_path.with_file_name("second.md");
    std::fs::write(&second_path, "second").expect("second fixture must be written");
    let second_path = second_path
        .canonicalize()
        .expect("second fixture path must be canonicalized");
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(second_path.clone()));
    harness.run_steps(5);
    second_path
}

fn active_document_buffer(
    harness: &egui_kittest::Harness<'static, katana_ui::shell::KatanaApp>,
) -> String {
    harness
        .state()
        .app_state_for_test()
        .active_document()
        .expect("active document should exist")
        .buffer
        .clone()
}

fn editor_config() -> EditorConfig {
    EditorConfig::new(EditorConfigInput {
        theme: theme(),
        strings: strings(),
        locale: Locale::new("en", TextDirection::Ltr),
        typography: Typography::new("Monospace", 14.0, 400, 1.4, 0.0),
        spacing: Spacing::new(8.0, 4.0, 6.0),
        settings: katana_language_editor::EditorSettings::new(
            AutosavePolicy::new(false, None),
            ShortcutMap::empty(),
            true,
            4,
            true,
            1.0,
        ),
        syntax_highlighter: Arc::new(NoopSyntaxHighlighter),
        clipboard: Arc::new(MemoryClipboard::new()),
    })
}

fn theme() -> Theme {
    let text = rgba(220, 224, 232);
    let muted = rgba(140, 148, 160);
    let background = rgba(28, 30, 34);
    Theme::new(
        ColorTokens::new(ColorTokensInput {
            text_primary: text,
            text_muted: muted,
            background,
            selection_background: rgba(54, 82, 130),
            caret: rgba(240, 240, 240),
            gutter_foreground: muted,
            current_line_background: rgba(36, 39, 45),
            hover_line_background: rgba(43, 48, 58),
            search_match_background: rgba(125, 102, 39),
            search_active_background: rgba(170, 126, 38),
            diagnostic_error: rgba(210, 74, 88),
            diagnostic_warning: rgba(214, 161, 58),
            diagnostic_info: rgba(92, 152, 214),
            decoration_accent: rgba(114, 156, 255),
        }),
        true,
    )
}

fn rgba(red: u8, green: u8, blue: u8) -> Rgba {
    Rgba::new(red, green, blue, 255)
}

fn strings() -> Strings {
    Strings::new(StringsInput {
        accessibility_label: "Editor".to_string(),
        context_copy: "Copy".to_string(),
        context_cut: "Cut".to_string(),
        context_paste: "Paste".to_string(),
        find_placeholder: "Find".to_string(),
        replace_placeholder: "Replace".to_string(),
        diagnostics_label: "Diagnostics".to_string(),
        diagnostic_fix_label: "Fix".to_string(),
        diagnostic_fix_all_label: "Fix all".to_string(),
        diagnostic_docs_label: "Docs".to_string(),
        read_only_label: "Read only".to_string(),
    })
}
```
