use super::{RULE, Visitor, is_target_file, patterns};
use std::path::{Path, PathBuf};
use syn::visit::Visit;

const CORE_CLIPBOARD: &str = "crates/katana-language-editor/src/clipboard.rs";
const CORE_CONTROLS: &str = "crates/katana-language-editor/src/controls.rs";
const EGUI_PASTE: &str = "crates/katana-language-editor-egui/src/clipboard_paste_control.rs";
const EGUI_MAPPING: &str = "crates/katana-language-editor-egui/src/kuc_text_surface_mapping.rs";
const EGUI_BINDING: &str = "crates/katana-language-editor-egui/src/kuc_text_surface_binding.rs";
const EGUI_OUTPUT: &str = "crates/katana-language-editor-egui/src/widget_output.rs";

fn lint(source: &str, path: &str) -> Result<Vec<crate::diagnostics::Violation>, syn::Error> {
    if !is_target_file(Path::new(path)) {
        return Ok(Vec::new());
    }
    let mut visitor = Visitor::new(PathBuf::from(path));
    visitor.visit_file(&syn::parse_file(source)?);
    Ok(visitor.into_violations())
}

fn assert_rejected(source: &str, path: &str, reason: &str) -> Result<(), syn::Error> {
    let violations = lint(source, path)?;
    assert_eq!(violations.len(), 1, "unexpected violations: {violations:?}");
    assert_eq!(violations[0].rule, RULE);
    assert!(violations[0].message.contains("KatanA host"));
    assert!(violations[0].message.contains(reason));
    Ok(())
}

#[test]
fn selects_only_clipboard_transaction_sources() {
    for path in [
        CORE_CLIPBOARD,
        CORE_CONTROLS,
        EGUI_PASTE,
        EGUI_MAPPING,
        EGUI_BINDING,
        EGUI_OUTPUT,
    ] {
        assert!(is_target_file(Path::new(path)), "not selected: {path}");
    }
    for path in [
        "crates/katana-language-editor/src/actions.rs",
        "crates/katana-language-editor-egui/src/host_controls.rs",
        "crates/katana-language-editor-egui/src/context-menu.rs",
        "crates/katana-language-editor-egui/src/platform_text_surface.rs",
        "crates/katana-language-editor-egui/src/kuc_text_surface_mapping_tests.rs",
    ] {
        assert!(
            !is_target_file(Path::new(path)),
            "unexpected target: {path}"
        );
    }
}

#[test]
fn rejects_arboard_import_and_path() -> Result<(), syn::Error> {
    for (source, path) in [
        ("use arboard;", EGUI_PASTE),
        ("use arboard as Clipboard;", EGUI_PASTE),
        ("use arboard::Clipboard;", EGUI_PASTE),
        ("fn f() { arboard::Clipboard::new(); }", EGUI_PASTE),
    ] {
        assert_rejected(source, path, "arboard clipboard access")?;
    }
    Ok(())
}

#[test]
fn rejects_katana_host_crates() -> Result<(), syn::Error> {
    for (source, path) in [
        ("use katana;", EGUI_OUTPUT),
        ("use katana_ui::Clipboard;", EGUI_OUTPUT),
        ("use katana_core::Clipboard;", EGUI_OUTPUT),
        ("use katana_platform::Clipboard;", EGUI_OUTPUT),
    ] {
        assert_rejected(source, path, "direct KatanA crate reference")?;
    }
    Ok(())
}

#[test]
fn rejects_aliased_katana_host_crates() -> Result<(), syn::Error> {
    for source in [
        "use katana as host;",
        "use katana_ui as host;",
        "use katana_core as host;",
        "use katana_platform as host;",
    ] {
        assert_rejected(source, EGUI_OUTPUT, "direct KatanA crate reference")?;
    }
    Ok(())
}

#[test]
fn rejects_macos_clipboard_processes() -> Result<(), syn::Error> {
    for command in ["osascript", "pbpaste", "pbcopy"] {
        let source = format!("fn acquire() {{ Command::new(\"{command}\"); }}");
        assert_rejected(&source, EGUI_PASTE, "macOS clipboard process invocation")?;
    }
    assert_rejected(
        "fn acquire() { std::process::Command::new(\"pbpaste\").arg(\"--raw\"); }",
        EGUI_PASTE,
        "macOS clipboard process invocation",
    )
}

#[test]
fn rejects_file_url_conversion_helpers_and_calls() -> Result<(), syn::Error> {
    for helper in [
        "file_url_to_path",
        "parse_file_url",
        "decode_file_url_path",
        "to_file_path",
    ] {
        let source = format!("fn {helper}() {{}}");
        assert_rejected(&source, EGUI_MAPPING, "file URL to path conversion")?;
    }
    assert_rejected(
        "fn acquire() { let _ = Url::parse(\"file:///tmp/image.png\"); }",
        EGUI_PASTE,
        "file:// URL parsing or conversion",
    )
}

#[test]
fn rejects_image_extension_decisions() -> Result<(), syn::Error> {
    for helper in ["supported_image_extension", "path_has_image_extension"] {
        let source = format!("fn {helper}() {{}}");
        assert_rejected(&source, EGUI_BINDING, "supported image extension decision")?;
    }
    Ok(())
}

#[test]
fn rejects_explicit_filesystem_acquisition_and_saving() -> Result<(), syn::Error> {
    for source in [
        "fn acquire() { std::fs::read(\"image.png\"); }",
        "fn acquire() { let _ = std::fs::File::open(\"image.png\"); }",
        "fn acquire() { let _ = std::fs::OpenOptions::new(); }",
        "fn save() { std::fs::write(\"image.png\", []); }",
        "fn acquire() { let _ = image::open(\"image.png\"); }",
        "fn save() { image::save_buffer(\"image.png\", &[], 1, 1, image::ColorType::Rgb8); }",
    ] {
        assert_rejected(
            source,
            EGUI_PASTE,
            "filesystem image-byte acquisition or saving",
        )?;
    }
    Ok(())
}

#[test]
fn allows_neutral_typed_intents_opaque_urls_and_generic_methods() -> Result<(), syn::Error> {
    for (source, path) in [
        (
            "fn map(action: EditorAction) { let EditorAction::IngestClipboardFileUrls { urls } = action else { return }; let _opaque: Vec<String> = urls; }",
            EGUI_OUTPUT,
        ),
        (
            "fn map() { let _urls = vec![\"file:///tmp/image.png\".to_string()]; }",
            EGUI_OUTPUT,
        ),
        (
            "fn map(editor: &mut Editor) { editor.read(); editor.write(); editor.save(); }",
            EGUI_PASTE,
        ),
        (
            "fn map() { let _ = katana_ui_core::text_surface::TextSurfaceEvent::default(); }",
            EGUI_BINDING,
        ),
    ] {
        assert!(lint(source, path)?.is_empty(), "unexpected violations");
    }
    Ok(())
}

#[test]
fn helper_matching_does_not_treat_opaque_urls_as_paths() {
    assert!(!patterns::is_file_url_helper("file_urls"));
    assert!(!patterns::is_image_extension_helper("image_name"));
}
