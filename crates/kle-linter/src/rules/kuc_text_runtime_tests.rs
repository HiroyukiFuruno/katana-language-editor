use super::KucTextRuntimeVisitor;
use syn::visit::Visit;

fn violations(
    fixture: &str,
    file: &str,
) -> Result<Vec<crate::diagnostics::Violation>, Box<dyn std::error::Error>> {
    let syntax = syn::parse_file(fixture)?;
    let mut visitor = KucTextRuntimeVisitor::new(file.into());
    visitor.visit_file(&syntax);
    Ok(visitor.into_violations())
}
#[test]
fn rejects_legacy_surface_types_inside_the_direct_show_path()
-> Result<(), Box<dyn std::error::Error>> {
    let violations = violations(POSITIVE_FIXTURE, "widget.rs")?;
    assert_eq!(violations.len(), 4, "unexpected violations: {violations:?}");
    assert!(
        violations
            .iter()
            .any(|value| value.message.contains("directly"))
    );
    Ok(())
}
#[test]
fn requires_a_direct_kuc_binding_show_call() -> Result<(), Box<dyn std::error::Error>> {
    let violations = violations(MISSING_BINDING_FIXTURE, "widget.rs")?;
    assert_eq!(violations.len(), 1, "unexpected violations: {violations:?}");
    assert!(
        violations
            .iter()
            .any(|value| value.message.contains("exactly once"))
    );
    Ok(())
}
#[test]
fn allows_legacy_types_outside_the_direct_show_path() -> Result<(), Box<dyn std::error::Error>> {
    let violations = violations(NEGATIVE_FIXTURE, "fixture.rs")?;

    assert!(
        violations.is_empty(),
        "unexpected violation: {:?}",
        violations
    );
    Ok(())
}
#[test]
fn rejects_local_gutter_reconstruction_and_requires_latest_kuc_frame()
-> Result<(), Box<dyn std::error::Error>> {
    let violations = violations(GUTTER_RECONSTRUCTION_FIXTURE, "gutter_control.rs")?;

    assert_eq!(violations.len(), 3, "unexpected violations: {violations:?}");
    assert!(
        violations
            .iter()
            .any(|violation| violation.message.contains("local gutter model"))
    );
    assert!(
        violations
            .iter()
            .any(|violation| violation.message.contains("enumerate source text"))
    );
    assert!(
        violations
            .iter()
            .any(|violation| violation.message.contains("latest KUC frame"))
    );
    Ok(())
}
#[test]
fn allows_gutter_control_that_consumes_latest_kuc_frame() -> Result<(), Box<dyn std::error::Error>>
{
    let violations = violations(KUC_FRAME_GUTTER_FIXTURE, "gutter_control.rs")?;

    assert!(
        violations.is_empty(),
        "unexpected violations: {violations:?}"
    );
    Ok(())
}
#[test]
fn rejects_a_separately_shown_command_chrome() -> Result<(), Box<dyn std::error::Error>> {
    let violations = violations(MISSING_COMPOSITION_FRAME_FIXTURE, "widget.rs")?;

    assert_eq!(violations.len(), 1, "unexpected violations: {violations:?}");
    assert!(violations[0].message.contains("root surface"));
    Ok(())
}
#[test]
fn rejects_show_with_chrome_in_the_public_show_path() -> Result<(), Box<dyn std::error::Error>> {
    let violations = violations(ROOT_COMPOSITION_FIXTURE, "widget.rs")?;
    assert_eq!(violations.len(), 2, "unexpected violations: {violations:?}");
    assert!(
        violations
            .iter()
            .any(|value| value.message.contains("show_with_chrome"))
    );
    Ok(())
}
#[test]
fn rejects_two_direct_kuc_binding_show_calls() -> Result<(), Box<dyn std::error::Error>> {
    let violations = violations(TWO_DIRECT_SHOWS_FIXTURE, "widget.rs")?;
    assert_eq!(violations.len(), 1, "unexpected violations: {violations:?}");
    assert!(violations[0].message.contains("exactly once"));
    Ok(())
}

#[test]
fn allows_one_direct_kuc_binding_show_call() -> Result<(), Box<dyn std::error::Error>> {
    assert!(violations(ONE_DIRECT_SHOW_FIXTURE, "widget.rs")?.is_empty());
    Ok(())
}
const POSITIVE_FIXTURE: &str = r#"
impl Demo {
    pub fn show(&mut self) {
        self.kuc_text_surface.show(ui);
        let _ = egui::TextEdit::multiline(&mut String::new());
        let _ = egui::ScrollArea::both();
        let _ = egui::Frame::new();
        let _ = PlatformTextSurface::new();
    }
}
"#;

const MISSING_BINDING_FIXTURE: &str = r#"
impl Demo {
    pub fn show(&mut self) {
        let _ = self;
    }
}
"#;

const NEGATIVE_FIXTURE: &str = r#"
const HOST_PRESENTATION_KEY: &str = "edit.bold";
const KUC_COMMAND_CHROME_OPAQUE_ID: &str = "kuc-command-bold";
static HOST_PRESENTATION_ID: &str = "kuc-command-bold";

fn render(ui: &mut ()) {
    let mut events: Vec<i32> = Vec::new();
    let label = format!("{}");
    let _ = label;
    events.push(1);
    let _ = ui;
}

fn sync() {
    let _ = String::from("kuc-text-surface");
}
"#;

const GUTTER_RECONSTRUCTION_FIXTURE: &str = r#"
impl Demo {
    fn gutter_lines(&self, source: &str) {
        let _ = LineGutterModel::build(source);
        let _ = source.lines().count();
    }
}
"#;

const KUC_FRAME_GUTTER_FIXTURE: &str = r#"
impl Demo {
    fn gutter_lines(&self) {
        let _ = self.latest_kuc_frame.as_ref().map(|record| &record.frame.gutter);
    }
}
"#;

const MISSING_COMPOSITION_FRAME_FIXTURE: &str = r#"
impl Demo {
    pub fn show(&mut self) {
        self.kuc_text_surface.show(ui);
        self.show_kuc_command_chrome(ui);
    }
}
"#;

const ROOT_COMPOSITION_FIXTURE: &str = r#"
impl Demo {
    pub fn show(&mut self) {
        self.kuc_text_surface.show_with_chrome(ui, request, config, chrome);
    }
}
"#;

const TWO_DIRECT_SHOWS_FIXTURE: &str = r#"
impl Demo {
    pub fn show(&mut self) {
        self.kuc_text_surface.show(ui);
        self.kuc_text_surface.show(ui);
    }
}
"#;

const ONE_DIRECT_SHOW_FIXTURE: &str = r#"
impl Demo {
    pub fn show(&mut self) {
        self.kuc_text_surface.show(ui);
    }
}
"#;
