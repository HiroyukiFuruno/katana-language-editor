pub(crate) const AUDIT_DOCUMENT: &str =
    include_str!("../../../docs/v0-1-0-katana-editor-full-parity-audit.md");

pub(crate) const RUNNABLE_BASELINE_COMMANDS: &[&str] = &[
    "cargo test -p katana-ui --test ui_integration_parallel editor_ui -- --list",
    "cargo test -p katana-ui --test ui_integration_parallel editor_lint_fix_review_button -- --list",
    "cargo test -p katana-ui --test ui_integration_parallel editor_ui::editor_context_menu_authoring_heading1_updates_editor_state_via_real_ui_routing -- --nocapture",
    "cargo test -p katana-ui --test ui_integration_parallel editor_ui::editor_context_menu_save_button_persists_editor_buffer_state -- --nocapture",
    "cargo test -p katana-ui --test ui_integration_parallel editor_ui::editor_context_menu_format_button_updates_formatted_markdown_buffer -- --nocapture",
    "cargo test -p katana-ui --test ui_integration_parallel editor_ui::editor_select_and_jump -- --nocapture",
    "cargo test -p katana-ui --test ui_integration_parallel editor_ui::test_integration_view_modes -- --nocapture",
    "cargo test -p katana-ui --test ui_integration_parallel editor_ui::test_integration_update_buffer -- --nocapture",
    "cargo test -p katana-ui --test ui_integration_parallel editor_ui::editor_scroll_sync -- --nocapture",
    "cargo test -p katana-ui --test ui_integration_parallel editor_navigation -- --nocapture",
    "cargo test -p katana-ui --test ui_integration_parallel editor_view_modes -- --nocapture",
    "cargo test -p katana-ui --test ui_integration_parallel editor_toggle_view_modes -- --nocapture",
    "cargo test -p katana-ui --test ui_integration_parallel editor_layout_persistence -- --nocapture",
    "cargo test -p katana-ui toolbar_popup -- --nocapture",
    "cargo test -p katana-ui --test ui_integration_parallel editor_ui::test_integration_editor_line_numbers_visibility -- --nocapture",
    "cargo test -p katana-ui --test ui_integration_parallel editor_ui::input_assist_code_block_button_updates_editor_buffer -- --nocapture",
    "cargo test -p katana-ui --test ui_integration_parallel editor_ui::code_block_kind_menu_closes_when_editor_is_clicked -- --nocapture",
    "cargo test -p katana-ui --test ui_integration_parallel editor_lint_fix_review_button::problems_status_count_follows_scope_only_while_panel_open -- --nocapture",
];

pub(crate) const SOURCE_OF_TRUTH_PATHS: &[&str] = &[
    "katana-ui/src/views/panels/editor/**",
    "katana-ui/tests/integration/editor/**",
    "katana-ui/src/app/action/process_authoring.rs",
    "katana-ui/src/app/document_edit.rs",
    "katana-ui/src/app/action/image_ingest.rs",
    "katana-ui/src/app/action/dispatch_secondary.rs",
    "katana-ui/src/editor_undo.rs",
];

pub(crate) const SOURCE_ONLY_EDITOR_MODULES: &[&str] = &["rendering.rs"];

pub(crate) const REFERENCE_ONLY_CONSTRAINTS: &[&str] = &[
    "## Evidence Revalidation (2026-08-13)",
    "Current authoritative status",
    "KatanA は変更しない",
    "historical planned path is not evidence",
    "crates/katana-ui/tests/integration/editor/kle_downstream_adapter.rs",
];
