use super::*;
use std::path::Path;
use syn::visit::Visit;

fn lint(source: &str) -> Vec<Violation> {
    let Ok(file) = syn::parse_file(source) else {
        return Vec::new();
    };
    let mut visitor = Visitor::new("crates/katana-language-editor/src/fixture.rs".into());
    visitor.visit_file(&file);
    visitor.violations
}

#[test]
fn accepts_opaque_projection_scenario_and_closed_receipt_transit() {
    let source = r#"
        use katana_ui_core::text_command_surface::{
            EguiTextCommandSurfaceHostProjectionLease,
            EguiTextCommandSurfaceRootEventDispatchReceipt,
            EguiTextCommandSurfaceRootEventForwardingReceipt,
            FullTextCommandSurfaceScenarioId,
        };
        use katana_ui_core::egui::OpaqueRootArtifactReceipt;
        use katana_language_editor_egui::KucRootBindingReceipt;
        fn forward(
            lease: EguiTextCommandSurfaceHostProjectionLease,
            scenario: FullTextCommandSurfaceScenarioId,
            dispatch: EguiTextCommandSurfaceRootEventDispatchReceipt,
            forwarding: EguiTextCommandSurfaceRootEventForwardingReceipt,
            artifact: OpaqueRootArtifactReceipt,
            receipt: KucRootBindingReceipt,
        ) {
            host.forward(lease, scenario, dispatch, forwarding, artifact, receipt);
        }
        fn generic_scenario() -> FullTextCommandSurfaceScenarioId {
            FullTextCommandSurfaceScenarioId::WorkspaceTabs
        }
    "#;
    assert!(
        lint(source).is_empty(),
        "unexpected violations: {:?}",
        lint(source)
    );
}

#[test]
fn accepts_unrelated_workspace_search_names() {
    assert!(lint("enum WorkspaceSearchOpen { Available } fn search() {}").is_empty());
}

#[test]
fn rejects_tab_and_group_internals() {
    for source in [
        "struct WorkspaceTabState { tab_id: String, group_index: usize }",
        "struct CloseableTabAdapter;",
        "struct SanitizedTabProjection;",
        "fn tab_event_port() {}",
        "use katana_ui_core::egui::text_command_surface::TabStripProposalPort;",
        "use katana_ui_core::egui::text_command_surface::TabStripContextMenuPresentation;",
        "use katana_ui_core::egui::text_command_surface::TabStripGroupPopupPresentation;",
        "use katana_ui_core::egui::text_command_surface::TabStripMenuOperation;",
        "fn group_name_input(value: String) { let _ = value; }",
    ] {
        assert!(!lint(source).is_empty(), "accepted: {source}");
    }
}

#[test]
fn rejects_direct_egui_tab_rendering_and_raw_colors() {
    for source in [
        "fn render_tabs(ui: &mut egui::Ui) { ui.button(\"Tab\"); }",
        "fn tab_view(ui: &mut egui::Ui) { ui.horizontal(|ui| { ui.label(\"x\"); }); }",
        "fn tab_colors() { let _ = \"#ff00aa\"; }",
    ] {
        assert!(!lint(source).is_empty(), "accepted: {source}");
    }
}

#[test]
fn does_not_inspect_cfg_test_modules() {
    let source = "#[cfg(test)] mod tests { struct WorkspaceTabState { tab_id: usize } }";
    assert!(
        lint(source).is_empty(),
        "unexpected violations: {:?}",
        lint(source)
    );
}

#[test]
fn targets_only_kle_runtime_and_storybook_paths() {
    assert!(patterns::is_target(Path::new(
        "crates/katana-language-editor/src/lib.rs"
    )));
    assert!(patterns::is_target(Path::new(
        "crates/katana-language-editor-egui/src/lib.rs"
    )));
    assert!(patterns::is_target(Path::new(
        "tools/kle-storybook/src/main.rs"
    )));
    assert!(!patterns::is_target(Path::new(
        "crates/katana-ui-core/src/lib.rs"
    )));
    assert!(patterns::is_test_source(Path::new(
        "tools/kle-storybook/src/tests.rs"
    )));
    assert!(patterns::is_test_source(Path::new(
        "crates/katana-language-editor/src/root_tests.rs"
    )));
}
