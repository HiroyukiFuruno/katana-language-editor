use super::*;
use syn::visit::Visit;

fn lint(source: &str) -> Vec<Violation> {
    let Ok(file) = syn::parse_file(source) else {
        return Vec::new();
    };
    let mut visitor = Visitor::new("fixture.rs".into());
    visitor.visit_file(&file);
    visitor.violations
}

#[test]
fn accepts_opaque_host_projection_lease_forwarding() {
    let source = r#"
        use katana_ui_core::text_command_surface::EguiTextCommandSurfaceHostProjectionLease;
        fn attach(lease: EguiTextCommandSurfaceHostProjectionLease) {
            host.attach_projection_lease(lease);
        }
    "#;
    assert!(
        lint(source).is_empty(),
        "unexpected violations: {:?}",
        lint(source)
    );
}

#[test]
fn rejects_fully_qualified_source_address_symbols() {
    for source in [
        "fn f() { let _ = katana_ui_core::text_command_surface::SourceAddressProjectionLease::new(); }",
        "fn f(value: katana_ui_core::text_command_surface::SourceAddressSubmission) { let _ = value; }",
        "fn f() { let _ = katana_ui_core::text_command_surface::EguiSourceAddressStripAdapter::new(); }",
        "enum SourceAddressEvent { Submitted }",
        "impl katana_ui_core::text_command_surface::SourceAddressStrip { fn render(&self) {} }",
    ] {
        assert!(!lint(source).is_empty(), "accepted: {source}");
    }
}

#[test]
fn rejects_imported_and_renamed_source_address_symbols() {
    for source in [
        "use katana_ui_core::text_command_surface::{SourceAddressStrip, SourceAddressEntry};",
        "use katana_ui_core::text_command_surface::SourceAddressSubmissionPort as Port;",
        "use katana_ui_core::text_command_surface::SourceAddressAction;",
    ] {
        assert!(!lint(source).is_empty(), "accepted: {source}");
    }
}

#[test]
fn rejects_url_action_and_file_path_construction() {
    for source in [
        "use url::Url; fn f(value: Url) { let _ = value; }",
        "fn f(value: Url) { let _ = value.join(\"child\"); }",
        "fn f() { let _ = Url::parse(\"file:///tmp/a.md\"); }",
        "fn f() { let _ = url::Url::from_file_path(\"/tmp/a.md\"); }",
        "struct UrlTabState { value: String }",
        "fn f() { let _ = OpenUrl(\"https://example.invalid\"); }",
        "fn parse_file_url(value: &str) { let _ = value; }",
        "fn f() { let _ = \"file:///tmp/a.md\"; }",
    ] {
        assert!(!lint(source).is_empty(), "accepted: {source}");
    }
}

#[test]
fn rejects_direct_egui_source_address_construction() {
    for source in [
        "fn render_source_address(ui: &mut egui::Ui) { ui.add(egui::TextEdit::singleline(&mut String::new())); }",
        "fn source_address_controls(ui: &mut egui::Ui) { ui.button(\"Open\"); }",
    ] {
        assert!(!lint(source).is_empty(), "accepted: {source}");
    }
}

#[test]
fn accepts_generic_egui_and_opaque_source_words_outside_forbidden_contracts() {
    let source = r#"
        fn render(ui: &mut egui::Ui) {
            ui.button("Generic");
        }
        fn forward_source_address(lease: EguiTextCommandSurfaceHostProjectionLease) {
            host.forward(lease);
        }
    "#;
    assert!(
        lint(source).is_empty(),
        "unexpected violations: {:?}",
        lint(source)
    );
}

#[test]
fn targets_only_kle_runtime_egui_and_storybook_paths() {
    assert!(is_target(Path::new(
        "crates/katana-language-editor/src/lib.rs"
    )));
    assert!(is_target(Path::new(
        "crates/katana-language-editor-egui/src/lib.rs"
    )));
    assert!(is_target(Path::new("tools/kle-storybook/src/main.rs")));
    assert!(!is_target(Path::new(
        "../katana-ui-core/crates/katana-ui-core/src/lib.rs"
    )));
}
