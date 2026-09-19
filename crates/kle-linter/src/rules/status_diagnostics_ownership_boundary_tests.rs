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
fn accepts_opaque_projection_and_closed_receipts() {
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
        ) { host.forward(lease, scenario, dispatch, forwarding, artifact, receipt); }
    "#;
    assert!(
        lint(source).is_empty(),
        "unexpected violations: {:?}",
        lint(source)
    );
}

#[test]
fn rejects_imports_definitions_and_fully_qualified_usage() {
    for source in [
        "use katana_ui_core::molecule::StatusBar;",
        "use katana_ui_core::molecule::DiagnosticsList as Problems;",
        "struct StatusBarState { diagnostics_list: DiagnosticsList }",
        "type Local = katana_ui_core::egui::EguiDiagnosticsListAdapter;",
        "fn render(ui: &mut egui::Ui) { let action = StatusBarAction::Activate; let _ = action; ui.label(\"x\"); }",
        "fn send(event: DiagnosticsListEvent) { let _ = event; }",
        "fn use_lease(lease: StatusDiagnosticsProjectionLease) { let _ = lease; }",
        "fn status_bar_render() {}",
    ] {
        assert!(!lint(source).is_empty(), "accepted: {source}");
    }
}

#[test]
fn rejects_renamed_imports_and_snake_case_state() {
    for source in [
        "use foo::StatusBarAction as LocalAction;",
        "use foo::DiagnosticsListEvent as LocalEvent;",
        "struct State { status_bar_visible: bool, diagnostics_list_items: usize }",
        "fn update(status_diagnostics_projection_lease: usize) { let _ = status_diagnostics_projection_lease; }",
    ] {
        assert!(!lint(source).is_empty(), "accepted: {source}");
    }
}

#[test]
fn does_not_inspect_cfg_test_modules() {
    let source = "#[cfg(test)] mod tests { struct StatusBarState { diagnostics_list: usize } }";
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
}
