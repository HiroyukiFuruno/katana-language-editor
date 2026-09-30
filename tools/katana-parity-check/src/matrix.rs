use crate::capability_manifest::{
    CapabilityManifest, CapabilityOwner, HostEffectKind, KatanaSourceEvidence,
    KleActualFrameHarness, KleActualInputEvidence, KleHostE2eEvidence, KleHostE2eTestLocator,
    KleSourceLocator,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvidenceClassification {
    ConfirmedKatana,
    UserMandatedAdditional,
}

pub(crate) struct FeatureVerification {
    pub(crate) id: &'static str,
    pub(crate) name: &'static str,
    pub(crate) automated_checks: &'static [&'static str],
    pub(crate) remaining_gap: &'static str,
    pub(crate) manifest: &'static CapabilityManifest,
}

impl FeatureVerification {
    pub(crate) fn has_open_gap(&self) -> bool {
        !self.remaining_gap.is_empty()
    }

    pub(crate) fn is_storybook_only(&self) -> bool {
        self.automated_checks
            .iter()
            .all(|check| check.contains("storybook"))
            && self.remaining_gap.is_empty()
    }
}

pub(crate) const HOST_E2E_CHECK: &str =
    "cargo test --manifest-path tools/katana-host-e2e/Cargo.toml --locked --test actual_host";
pub(crate) const REQUIRED_FEATURE_COUNT: usize = 26;

const HOST_E2E_SOURCE: &str = "tools/katana-host-e2e/tests/actual_host.rs";
const fn missing_host_e2e(selector: &'static str) -> KleHostE2eEvidence {
    KleHostE2eEvidence {
        test: KleHostE2eTestLocator {
            target: "actual_host",
            source_path: HOST_E2E_SOURCE,
            selector,
        },
        effect: HostEffectKind::Missing,
    }
}
const KLE_ACTUAL_FRAME_HARNESS: KleActualFrameHarness = KleActualFrameHarness {
    public_show_callsite: KleSourceLocator {
        source_path: "crates/katana-language-editor-egui/src/direct_text_surface_test_support.rs",
        line: 139,
        marker: "context.run_ui(input(events), |ui| result = Some(editor.show(ui)))",
    },
    raw_input_root: KleSourceLocator {
        source_path: "crates/katana-language-editor-egui/src/direct_text_surface_test_support.rs",
        line: 187,
        marker: "fn input(events: Vec<Event>) -> RawInput {",
    },
    raw_input_construction: KleSourceLocator {
        source_path: "crates/katana-language-editor-egui/src/direct_text_surface_test_support.rs",
        line: 188,
        marker: "RawInput {",
    },
    scenario_implementation: KleSourceLocator {
        source_path: "crates/katana-language-editor-egui/src/direct_text_surface_test_support.rs",
        line: 27,
        marker: "impl DirectTextSurfaceScenario {",
    },
    symbol: "DirectTextSurfaceScenario",
    call_path: "DirectTextSurfaceScenario::run",
};
const REQUIRED_OWNERS: &[CapabilityOwner] = &[
    CapabilityOwner::KucRuntime,
    CapabilityOwner::KleBinding,
    CapabilityOwner::KatanaHost,
];

macro_rules! manifest {
    ($feature:literal, $classification:expr, $kle_source:literal, $kle_selector:literal, $source:literal, $line:literal, $marker:literal, $host_selector:literal) => {
        &CapabilityManifest {
            classification: $classification,
            katana_sources: &[KatanaSourceEvidence {
                path: $source,
                line: $line,
                marker: $marker,
            }],
            owners: REQUIRED_OWNERS,
            kle_actual_input: KleActualInputEvidence {
                feature_id: $feature,
                source_path: $kle_source,
                selector: $kle_selector,
                harness: KLE_ACTUAL_FRAME_HARNESS,
            },
            host_e2e: missing_host_e2e($host_selector),
        }
    };
}

macro_rules! feature {
    ($id:literal, $name:literal, $classification:expr, $kle_source:literal, $kle_selector:literal, $source:literal, $line:literal, $marker:literal, $test:literal, $gap:literal) => {
        FeatureVerification {
            id: $id,
            name: $name,
            automated_checks: &[
                "cargo test -p katana-language-editor-egui --locked",
                "just storybook-contract-check",
                HOST_E2E_CHECK,
            ],
            remaining_gap: $gap,
            manifest: manifest!(
                $id,
                $classification,
                $kle_source,
                $kle_selector,
                $source,
                $line,
                $marker,
                $test
            ),
        }
    };
}

#[rustfmt::skip]
pub(crate) const FEATURES: &[FeatureVerification] = &[
    feature!("text-editing", "multiline editing and read-only/reference blocking", EvidenceClassification::ConfirmedKatana, "crates/katana-language-editor-egui/src/direct_text_surface_integration_tests.rs", "public_show_text_editing_updates_buffer_and_honors_read_only", "crates/katana-ui/src/views/panels/editor/text_edit.rs", 51, "egui::TextEdit::multiline(buffer)", "kle_text_editing_updates_buffer", "missing actual KLE host-effect E2E"),
    feature!("document-scoped-undo", "workspace/document-scoped identity and external undo", EvidenceClassification::ConfirmedKatana, "crates/katana-language-editor-egui/src/document_control_tests.rs", "public_show_document_scoped_undo_tracks_external_changes", "crates/katana-ui/src/editor_undo.rs", 37, "undoer.add_undo", "kle_document_scoped_undo", "missing actual KLE host-effect E2E"),
    feature!("cursor-selection-restore", "cursor selection and pending cursor restore", EvidenceClassification::ConfirmedKatana, "crates/katana-language-editor-egui/src/direct_text_surface_integration_tests.rs", "public_show_cursor_selection_restores_after_authoring", "crates/katana-ui/src/app/action/process_authoring.rs", 55, "self.pending_editor_cursor", "kle_cursor_restore", "missing actual KLE host-effect E2E"),
    feature!("line-gutter", "line numbers, active line, hover line, clickable line numbers", EvidenceClassification::ConfirmedKatana, "crates/katana-language-editor-egui/src/gutter_control_tests.rs", "public_show_line_gutter_projects_active_hover_and_markers", "crates/katana-ui/src/views/panels/editor/line_numbers.rs", 28, "pub(crate) fn render", "kle_line_gutter", "missing actual KLE host-effect E2E"),
    feature!("diagnostics-problems", "diagnostic gutter popup, lint fix actions, and Problems panel", EvidenceClassification::ConfirmedKatana, "crates/katana-language-editor-egui/src/diagnostic_popup_tests.rs", "public_show_diagnostics_problems_routes_actual_actions", "crates/katana-ui/src/views/panels/editor/diagnostics_ui.rs", 7, "pub(crate) fn render_diagnostics", "kle_diagnostics_problems", "missing actual KLE host-effect E2E"),
    feature!("authoring-toolbar", "authoring toolbar popup anchored to cursor", EvidenceClassification::ConfirmedKatana, "crates/katana-language-editor-egui/src/kuc_command_chrome_composition_f2_tests.rs", "public_show_authoring_toolbar_maps_all_commands_and_image", "crates/katana-ui/src/views/panels/editor/toolbar.rs", 59, "AppAction::AuthorMarkdown", "kle_authoring_toolbar", "missing actual KLE host-effect E2E"),
    feature!("markdown-authoring-ops", "markdown authoring operation inventory", EvidenceClassification::ConfirmedKatana, "crates/katana-language-editor-egui/src/authoring_control_tests.rs", "public_show_markdown_authoring_transforms_all_commands", "crates/katana-ui/src/views/panels/editor/authoring.rs", 25, "match op", "kle_markdown_authoring", "missing actual KLE host-effect E2E"),
    feature!("code-block-menu", "code block kind menu lifecycle", EvidenceClassification::ConfirmedKatana, "crates/katana-language-editor-egui/src/kuc_command_chrome_composition_f2_tests.rs", "public_show_code_block_menu_maps_all_kinds_and_lifecycle", "crates/katana-ui/src/views/panels/editor/code_block_menu.rs", 22, "CodeBlockKind::all", "kle_code_block_menu", "missing actual KLE host-effect E2E"),
    feature!("context-menu-actions", "editor context menu save, format, authoring, and ingest actions", EvidenceClassification::ConfirmedKatana, "crates/katana-language-editor-egui/src/context_menu_actual_input_tests.rs", "public_show_context_menu_routes_save_format_authoring_and_ingest", "crates/katana-ui/src/views/panels/editor/context_menu.rs", 142, "MenuButtonOps::show", "kle_context_menu", "missing complete actual KLE host-effect E2E"),
    feature!("image-ingest", "image ingest from file, clipboard image, and clipboard file URL", EvidenceClassification::ConfirmedKatana, "crates/katana-language-editor-egui/src/image_ingest_actual_input_tests.rs", "public_show_image_ingest_routes_file_clipboard_and_file_url", "crates/katana-ui/src/app/action/image_ingest.rs", 61, "process_image_ingest", "kle_image_ingest", "missing clipboard file URL actual KLE host-effect E2E"),
    feature!("document-search", "document search highlighting and navigation", EvidenceClassification::ConfirmedKatana, "crates/katana-language-editor-egui/src/kuc_command_chrome_composition_f2_tests.rs", "public_show_document_search_updates_highlights_navigation_and_scroll", "crates/katana-ui/src/app/doc_search.rs", 10, "doc_search_matches", "missing_actual_kle_document_search_host_effect_coverage", "missing actual KLE host-effect E2E"),
    feature!("scroll-sync", "preview/editor scroll sync and source ownership", EvidenceClassification::ConfirmedKatana, "crates/katana-language-editor-egui/src/direct_text_surface_integration_tests.rs", "public_show_scroll_sync_preserves_source_ownership", "crates/katana-ui/src/views/panels/editor/logic_scroll.rs", 88, "pub fn update_scroll_sync", "kle_scroll_sync", "missing actual KLE host-effect E2E"),
    feature!("view-modes", "PreviewOnly, Split, CodeOnly and split direction", EvidenceClassification::ConfirmedKatana, "crates/katana-language-editor-egui/src/navigation_control_tests.rs", "public_show_view_modes_and_split_direction", "crates/katana-ui/src/views/panels/editor/layout.rs", 14, "pub fn layout", "kle_view_modes", "missing actual KLE host-effect E2E"),
    feature!("select-and-jump", "select document and jump to line with PreviewOnly fallback", EvidenceClassification::ConfirmedKatana, "crates/katana-language-editor-egui/src/navigation_control_tests.rs", "public_show_select_and_jump_handles_preview_only", "crates/katana-ui/src/views/panels/editor/logic_scroll.rs", 13, "toc_scroll_to_line", "kle_select_and_jump", "missing actual KLE host-effect E2E"),
    feature!("dirty-save-refresh", "dirty state, save clean, preview/search/diagnostics refresh", EvidenceClassification::ConfirmedKatana, "crates/katana-language-editor-egui/src/document_control_tests.rs", "public_show_dirty_save_refreshes_host_state", "crates/katana-ui/src/app/document_edit.rs", 40, "doc.update_buffer", "kle_dirty_save_refresh", "missing complete actual KLE host-effect E2E"),
    feature!("multi-document-state", "multi-document tab navigation and scoped editor state", EvidenceClassification::ConfirmedKatana, "crates/katana-language-editor-egui/src/document_control_tests.rs", "public_show_multi_document_state_is_scoped", "crates/katana-ui/src/editor_undo.rs", 32, "EditorUndoIdentity::text_edit_id", "kle_multi_document_state", "missing actual KLE host-effect E2E"),
    feature!("kuc-text-input", "KUC-backed emoji, IME, text area behavior, and OS font fallback", EvidenceClassification::ConfirmedKatana, "crates/katana-language-editor-egui/src/kuc_text_surface_render_tests.rs", "public_show_kuc_text_input_renders_japanese_star_and_ime", "crates/katana-ui/src/views/panels/editor/text_edit.rs", 51, "egui::TextEdit::multiline(buffer)", "kle_kuc_text_input", "missing actual KLE host-effect E2E"),
    feature!("syntax-highlighting", "host-injected syntax highlighting reaches KatanA layout", EvidenceClassification::ConfirmedKatana, "crates/katana-language-editor-egui/src/highlight_layout.rs", "public_show_syntax_highlighting_uses_injected_provider", "crates/katana-ui/src/views/panels/editor/layout.rs", 41, "if let Some(syntax)", "kle_syntax_highlighting", "missing actual KLE host-effect E2E"),
    feature!("clipboard-text", "text clipboard copy cut paste and read-only/error behavior", EvidenceClassification::ConfirmedKatana, "crates/katana-language-editor-egui/src/host_controls_tests.rs", "public_show_clipboard_text_honors_read_only", "crates/katana-ui/src/views/panels/editor/paste.rs", 48, "egui::Event::Paste(text)", "kle_clipboard_text", "missing actual KLE host-effect E2E"),
    feature!("editor-shortcuts", "shortcut mapping command routing and conflict rejection", EvidenceClassification::ConfirmedKatana, "crates/katana-language-editor-egui/src/host_controls_tests.rs", "public_show_shortcuts_route_and_reject_conflicts", "crates/katana-ui/src/views/panels/editor/logic_tests.rs", 203, "command_v_after_text_edit", "kle_editor_shortcuts", "missing actual KLE host-effect E2E"),
    feature!("native-text-input", "native multi-byte selection and IME composition", EvidenceClassification::ConfirmedKatana, "crates/katana-language-editor-egui/src/direct_text_surface_integration_tests.rs", "public_show_uses_kuc_surface_for_unicode_ime_focus_scroll_and_artifact", "crates/katana-ui/src/views/panels/editor/text_edit.rs", 51, "egui::TextEdit::multiline(buffer)", "kle_native_text_input", "missing actual KLE host-effect E2E"),
    feature!("text-surface", "shared KUC text surface, selection, IME, gutter, annotations, and same-frame artifact", EvidenceClassification::ConfirmedKatana, "crates/katana-language-editor-egui/src/direct_text_surface_integration_tests.rs", "public_show_text_surface_projects_selection_gutter_annotations_and_artifact", "crates/katana-ui/src/views/panels/editor/text_edit.rs", 74, "text_output.cursor_range", "kle_text_surface", "missing actual KLE host-effect E2E"),
    feature!("command-chrome", "shared command chrome for cursor toolbar, code menu, icons, and find/replace controls", EvidenceClassification::ConfirmedKatana, "crates/katana-language-editor-egui/src/kuc_command_chrome_composition_f2_tests.rs", "public_show_command_chrome_maps_toolbar_menu_search_and_replace", "crates/katana-ui/src/views/panels/editor/toolbar_popup.rs", 65, "EditorToolbar::new", "kle_command_chrome", "missing complete actual KLE host-effect E2E"),
    feature!("editing-history", "undo/redo and history shortcut behavior", EvidenceClassification::ConfirmedKatana, "crates/katana-language-editor-egui/src/host_controls_tests.rs", "public_show_editing_history_undo_redo_behavior", "crates/katana-ui/src/editor_undo.rs", 37, "undoer.add_undo", "kle_editing_history", "missing actual KLE host-effect E2E"),
    feature!("accessibility-tree", "AccessKit editor tree for text, selection, read-only, gutter, and menus", EvidenceClassification::ConfirmedKatana, "crates/katana-language-editor-egui/src/direct_text_surface_integration_tests.rs", "public_show_accessibility_tree_covers_editor_gutter_and_menus", "crates/katana-ui/src/views/panels/editor/ui.rs", 51, "pub fn show", "kle_accessibility_tree", "missing actual KLE host-effect E2E"),
    feature!("find-replace-ui", "find and replace user interface including replace-all", EvidenceClassification::UserMandatedAdditional, "crates/katana-language-editor-egui/src/kuc_command_chrome_composition_f2_tests.rs", "public_show_find_replace_ui_maps_search_replace_and_close", "crates/katana-ui/src/views/top_bar/search.rs", 13, "pub fn show", "missing_actual_kle_replace_host_effect_coverage", "user-mandated replace/replace-all lacks actual KLE host-effect E2E"),
];

#[cfg(test)]
pub(crate) const TEST_MANIFEST: CapabilityManifest = CapabilityManifest {
    classification: EvidenceClassification::ConfirmedKatana,
    katana_sources: &[KatanaSourceEvidence {
        path: "fixture.rs",
        line: 1,
        marker: "fixture",
    }],
    owners: REQUIRED_OWNERS,
    kle_actual_input: KleActualInputEvidence {
        feature_id: "fixture",
        source_path: "crates/katana-language-editor-egui/src/direct_text_surface_integration_tests.rs",
        selector: "public_show_uses_kuc_surface_for_unicode_ime_focus_scroll_and_artifact",
        harness: KLE_ACTUAL_FRAME_HARNESS,
    },
    host_e2e: KleHostE2eEvidence {
        test: KleHostE2eTestLocator {
            target: "actual_host",
            source_path: HOST_E2E_SOURCE,
            selector: "actual_kle_context_menu_authoring_raw_input_changes_katana_document",
        },
        effect: HostEffectKind::ContextAuthoring,
    },
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability_manifest::CapabilityManifestAudit;

    #[test]
    fn matrix_enforces_26_unique_fail_closed_capabilities() -> Result<(), String> {
        assert_eq!(FEATURES.len(), REQUIRED_FEATURE_COUNT);
        CapabilityManifestAudit::validate_feature_set(FEATURES)?;
        for (index, feature) in FEATURES.iter().enumerate() {
            assert!(!feature.id.is_empty());
            assert!(FEATURES[..index].iter().all(|prior| prior.id != feature.id));
            assert!(
                feature.has_open_gap(),
                "missing explicit gap: {}",
                feature.id
            );
            assert_eq!(feature.manifest.katana_sources.len(), 1);
            assert!(feature.manifest.katana_sources[0].line > 0);
            assert!(!feature.manifest.katana_sources[0].marker.is_empty());
            assert_eq!(feature.manifest.owners, REQUIRED_OWNERS);
            assert_eq!(feature.manifest.kle_actual_input.feature_id, feature.id);
            assert!(
                feature
                    .manifest
                    .kle_actual_input
                    .selector
                    .starts_with("public_show_")
            );
            assert_eq!(
                feature
                    .manifest
                    .kle_actual_input
                    .harness
                    .public_show_callsite
                    .source_path,
                KLE_ACTUAL_FRAME_HARNESS.public_show_callsite.source_path
            );
            assert!(
                feature
                    .manifest
                    .kle_actual_input
                    .harness
                    .public_show_callsite
                    .line
                    > 0
            );
            assert_eq!(
                feature
                    .manifest
                    .kle_actual_input
                    .harness
                    .scenario_implementation
                    .source_path,
                KLE_ACTUAL_FRAME_HARNESS.scenario_implementation.source_path
            );
            assert!(
                feature
                    .manifest
                    .kle_actual_input
                    .harness
                    .scenario_implementation
                    .line
                    > 0
            );
            assert_eq!(
                feature.manifest.kle_actual_input.harness.symbol,
                "DirectTextSurfaceScenario"
            );
            assert_eq!(
                feature.manifest.kle_actual_input.harness.call_path,
                "DirectTextSurfaceScenario::run"
            );
            assert!(
                feature
                    .manifest
                    .kle_actual_input
                    .harness
                    .raw_input_root
                    .line
                    > 0
            );
            assert!(
                feature
                    .manifest
                    .kle_actual_input
                    .harness
                    .raw_input_construction
                    .line
                    > 0
            );
        }
        Ok(())
    }

    #[test]
    fn user_mandated_replace_remains_in_the_required_matrix() -> Result<(), String> {
        let replace = FEATURES
            .iter()
            .find(|feature| feature.id == "find-replace-ui")
            .ok_or_else(|| "find-replace-ui must exist in the required matrix".to_string())?;
        assert_eq!(
            replace.manifest.classification,
            EvidenceClassification::UserMandatedAdditional
        );
        assert!(replace.has_open_gap());
        Ok(())
    }

    #[test]
    fn user_mandated_replace_leaf_is_separate_from_katana_find_navigation() -> Result<(), String> {
        let replace = FEATURES
            .iter()
            .find(|feature| feature.id == "find-replace-ui")
            .ok_or_else(|| "find-replace-ui must exist in the required matrix".to_string())?;
        let find = FEATURES
            .iter()
            .find(|feature| feature.id == "document-search")
            .ok_or_else(|| "document-search must exist in the required matrix".to_string())?;
        assert_ne!(replace.id, find.id);
        assert_ne!(
            replace.manifest.kle_actual_input.selector,
            find.manifest.kle_actual_input.selector
        );
        assert_ne!(
            replace.manifest.host_e2e.test.selector,
            find.manifest.host_e2e.test.selector
        );
        assert!(replace.has_open_gap());
        assert!(replace.remaining_gap.contains("replace"));
        Ok(())
    }

    #[test]
    fn historical_matrix_shape_symbols_remain_test_reachable() {
        let feature = &FEATURES[0];
        assert!(!feature.name.is_empty());
        assert!(!feature.automated_checks.is_empty());
        assert!(!feature.is_storybook_only());
        assert_eq!(
            TEST_MANIFEST.classification,
            EvidenceClassification::ConfirmedKatana
        );
    }
}
