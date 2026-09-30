use super::CommandChromeEventMappingVisitor;
use syn::visit::Visit;

#[test]
fn rejects_f2_mapping_boundary_violations_and_requires_kuc_delegation()
-> Result<(), Box<dyn std::error::Error>> {
    let mut mapping = CommandChromeEventMappingVisitor::new("widget.rs".into());
    mapping.visit_file(&syn::parse_file(MAPPING_FIXTURE)?);
    let mut scroll = CommandChromeEventMappingVisitor::new("search_control.rs".into());
    scroll.visit_file(&syn::parse_file(SCROLL_FIXTURE)?);
    let violations = [mapping.into_violations(), scroll.into_violations()].concat();
    assert_eq!(violations.len(), 8, "unexpected violations: {violations:?}");
    assert!(
        violations
            .iter()
            .any(|value| value.message.contains("typed target map"))
    );
    assert!(
        violations
            .iter()
            .any(|value| value.message.contains("scroll_into_view"))
    );
    Ok(())
}

#[test]
fn accepts_root_output_mapping_and_kuc_search_scroll() -> Result<(), Box<dyn std::error::Error>> {
    let mut mapping = CommandChromeEventMappingVisitor::new("widget.rs".into());
    mapping.visit_file(&syn::parse_file(POSITIVE_MAPPING_FIXTURE)?);
    let mut scroll = CommandChromeEventMappingVisitor::new("search_control.rs".into());
    scroll.visit_file(&syn::parse_file(POSITIVE_SCROLL_FIXTURE)?);
    assert!(mapping.into_violations().is_empty());
    assert!(scroll.into_violations().is_empty());
    Ok(())
}

#[test]
fn requires_exactly_one_root_output_mapping() -> Result<(), Box<dyn std::error::Error>> {
    let mut missing = CommandChromeEventMappingVisitor::new("widget.rs".into());
    missing.visit_file(&syn::parse_file(ROOT_WITHOUT_MAPPING_FIXTURE)?);
    let mut duplicate = CommandChromeEventMappingVisitor::new("widget.rs".into());
    duplicate.visit_file(&syn::parse_file(ROOT_WITH_DUPLICATE_MAPPING_FIXTURE)?);
    assert_eq!(missing.into_violations().len(), 1);
    assert_eq!(duplicate.into_violations().len(), 1);
    Ok(())
}

const MAPPING_FIXTURE: &str = r#"
struct SearchForm { query: String }
impl Demo {
    fn show(&mut self, id: String) {
        let _ = egui::Area::new("toolbar");
        match id.as_str() { "host" => (), _ => () }
        self.source.lines();
    }
}
"#;

const SCROLL_FIXTURE: &str = r#"
impl Demo {
    fn scroll_to_active_search_match(&mut self) {
        self.line_for_offset();
        self.scroll_to_line();
    }
}
"#;

const POSITIVE_MAPPING_FIXTURE: &str = r#"
impl Demo {
    fn show(&mut self, output: Output) {
        self.map_kuc_command_chrome_output(&output);
    }
}
"#;

const POSITIVE_SCROLL_FIXTURE: &str = r#"
impl Demo {
    fn scroll_to_active_search_match(&mut self) {
        EditorScrollControl::scroll_into_view(self, range);
    }
}
"#;

const ROOT_WITHOUT_MAPPING_FIXTURE: &str = r#"
impl Demo {
    fn show(&mut self) {}
}
"#;

const ROOT_WITH_DUPLICATE_MAPPING_FIXTURE: &str = r#"
impl Demo {
    fn show(&mut self, output: Output) {
        self.map_kuc_command_chrome_output(&output);
        self.map_kuc_command_chrome_output(&output);
    }
}
"#;
