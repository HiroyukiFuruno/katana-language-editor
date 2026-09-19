use super::CommandChromeRuntimeVisitor;
use syn::visit::Visit;

#[test]
fn rejects_composition_fallbacks_and_allows_root_surface_composition()
-> Result<(), Box<dyn std::error::Error>> {
    let syntax = syn::parse_file(FIXTURE)?;
    let mut visitor = CommandChromeRuntimeVisitor::new("kuc_command_chrome_composition.rs".into());
    visitor.visit_file(&syntax);
    let violations = visitor.into_violations();

    assert_eq!(violations.len(), 6, "unexpected violations: {violations:?}");
    assert!(
        violations
            .iter()
            .all(|value| value.message.contains("command chrome"))
    );
    Ok(())
}

#[test]
fn allows_root_surface_presentation() -> Result<(), Box<dyn std::error::Error>> {
    let syntax = syn::parse_file(ROOT_PRESENTATION_FIXTURE)?;
    let mut visitor = CommandChromeRuntimeVisitor::new("kuc_command_chrome_composition.rs".into());
    visitor.visit_file(&syntax);
    assert!(visitor.into_violations().is_empty());
    Ok(())
}

const FIXTURE: &str = r#"
impl Demo {
    fn show(&mut self) {
        let _ = egui::Area::new("fallback");
        let _ = egui::TextEdit::singleline(&mut String::new());
        let _ = UiRect::default();
        let _ = RawInput::default();
        self.pointer_button();
        self.button();
    }
}
"#;

const ROOT_PRESENTATION_FIXTURE: &str = r#"
impl Demo {
    fn root_presentation(&self) -> EguiTextCommandSurfacePresentation {
        EguiTextCommandSurfacePresentation::default()
    }
}
"#;
