use super::source_requirements_ledger::SourceRequirementsLedger;

#[test]
fn requirement_source_requires_direct_or_directory_universe_coverage() {
    assert!(
        SourceRequirementsLedger::validate_coverage(
            b"## Editor Parity Requirements\ncrates/katana-ui/src/views/modals/search_tabs/filename_tab.rs",
            b"crates/katana-ui/src/views/modals/search_tabs/**",
        )
        .is_ok()
    );
    assert!(matches!(
        SourceRequirementsLedger::validate_coverage(
            b"## Editor Parity Requirements\ncrates/katana-ui/src/views/modals/search_tabs/filename_tab.rs",
            b"crates/katana-ui/src/views/panels/editor/**",
        ),
        Err(error) if error.contains("search_tabs/filename_tab.rs")
    ));
}

#[test]
fn requirement_source_normalizes_only_katana_ui_namespaces() {
    let sources = SourceRequirementsLedger::editor_requirement_source_paths_for_test(
        "app_frame/breadcrumbs.rs top_bar/status_bar.rs preview_pane/slideshow/modal.rs \\
         katana-ui-core/src/render_model/typed_text.rs toolbar_popup.rs",
    );

    assert!(sources.contains("crates/katana-ui/src/views/app_frame/breadcrumbs.rs"));
    assert!(sources.contains("crates/katana-ui/src/views/top_bar/status_bar.rs"));
    assert!(sources.contains("crates/katana-ui/src/preview_pane/slideshow/modal.rs"));
    assert_eq!(sources.len(), 3);
}
