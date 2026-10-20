use super::ContextMenuRuntimeRule;
use crate::workspace::WorkspaceModel;
use std::path::Path;
use syn::visit::Visit;

fn violations(source: &str) -> Vec<crate::diagnostics::Violation> {
    violations_in("kuc_context_menu_host_presentation.rs", source)
}

fn violations_in(file: &str, source: &str) -> Vec<crate::diagnostics::Violation> {
    let Ok(syntax) = syn::parse_file(source) else {
        return Vec::new();
    };
    let mut visitor = super::Visitor::new(file.into());
    visitor.visit_file(&syntax);
    visitor.violations
}

#[test]
fn rejects_local_adapter_geometry_and_clipboard_file_url_leaf() {
    let found = violations(
        "fn f() { let _ = EguiContextMenuAdapter::default(); let _ = egui::Area::new(\"x\"); let _ = KucContextMenuItem::new(\"x\", \"x\").with_target(IngestClipboardFileUrls); }",
    );
    assert_eq!(found.len(), 3);
}

#[test]
fn allows_root_projection_and_neutral_context_targets() {
    assert!(violations("fn f() { let _ = KucContextMenuItem::new(\"save\", \"保存\").with_target(EditorAction::SaveDocument); }").is_empty());
}

#[test]
fn rejects_forbidden_calls_in_each_context_integration_file() {
    for file in [
        "kuc_context_menu_host_validation.rs",
        "kuc_context_menu_binding.rs",
        "kuc_context_menu_composition.rs",
        "kuc_text_surface_binding.rs",
        "kuc_artifact_aggregate.rs",
        "widget.rs",
        "widget_state.rs",
    ] {
        assert_eq!(
            violations_in(file, "fn f() { let _ = adapter.artifact_paint_plans(); }\n").len(),
            1,
            "{file}"
        );
    }
}

#[test]
fn rejects_context_geometry_and_root_setters() {
    let found = violations(
        "fn f() { let _ = ContextMenuPlacement::default(); root.set_context_menu(x); let _ = egui::Area::new(\"x\"); }",
    );
    assert_eq!(found.len(), 3);
}

#[test]
fn selector_type_is_registered() {
    let _ = ContextMenuRuntimeRule;
    let _ = WorkspaceModel::load(Path::new(".")).is_ok();
}
