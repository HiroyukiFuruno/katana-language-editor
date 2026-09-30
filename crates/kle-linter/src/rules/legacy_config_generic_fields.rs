use super::architecture::LIB_CRATE;
use crate::diagnostics::{KleLintError, Violation};
use crate::span::SpanOps;
use crate::workspace::WorkspaceModel;
use std::path::{Path, PathBuf};
use syn::ext::IdentExt;
use syn::visit::Visit;

const RULE: &str = "legacy-config-generic-field";
const STYLE_RULE: &str = "legacy-ui-style-definition";
const SETTINGS_RULE: &str = "legacy-settings-definition";
const STRUCTS: [&str; 2] = ["EditorConfig", "EditorConfigInput"];
const FIELDS: [&str; 6] = [
    "typography",
    "spacing",
    "settings",
    "theme",
    "strings",
    "locale",
];
const STYLE_DEFINITIONS: [&str; 7] = [
    "Typography",
    "Spacing",
    "Rgba",
    "ColorTokens",
    "ColorTokensInput",
    "Theme",
    "EditorTheme",
];
const SETTINGS_DEFINITIONS: [&str; 7] = [
    "EditorSettings",
    "AutosavePolicy",
    "ShortcutMap",
    "ShortcutBinding",
    "KeyBinding",
    "KeyModifier",
    "SemanticAction",
];

pub(super) struct LegacyConfigGenericFieldsRule;

impl LegacyConfigGenericFieldsRule {
    pub(super) fn check(workspace: &WorkspaceModel) -> Result<Vec<Violation>, KleLintError> {
        let mut violations = Vec::new();
        for file in workspace.rust_files() {
            if !is_neutral_source(workspace.root(), file.path()) {
                continue;
            }
            let mut visitor = Visitor::new(file.path().to_path_buf());
            visitor.visit_file(file.syntax());
            violations.extend(visitor.into_violations());
        }
        Ok(violations)
    }
}

fn is_neutral_source(root: &Path, path: &Path) -> bool {
    path.starts_with(root.join(LIB_CRATE).join("src"))
}

struct Visitor {
    file: PathBuf,
    violations: Vec<Violation>,
}

impl Visitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }

    fn into_violations(self) -> Vec<Violation> {
        self.violations
    }

    fn check_field(&mut self, field: &syn::Field) {
        let Some(ident) = &field.ident else {
            return;
        };
        let name = ident.unraw().to_string();
        if !FIELDS.contains(&name.as_str()) {
            return;
        }
        let location = SpanOps::start(ident.span());
        self.violations.push(Violation::new(
            self.file.clone(),
            location.line,
            location.column,
            RULE,
            format!("{name} must not be retained in the legacy editor config."),
        ));
    }

    fn check_style_definition(&mut self, ident: &syn::Ident) {
        let name = ident.unraw().to_string();
        if !STYLE_DEFINITIONS.contains(&name.as_str()) {
            return;
        }
        let location = SpanOps::start(ident.span());
        self.violations.push(Violation::new(
            self.file.clone(),
            location.line,
            location.column,
            STYLE_RULE,
            format!("{name} must not be retained as a legacy UI style definition."),
        ));
    }

    fn check_settings_definition(&mut self, ident: &syn::Ident) {
        let name = ident.unraw().to_string();
        if !SETTINGS_DEFINITIONS.contains(&name.as_str()) {
            return;
        }
        let location = SpanOps::start(ident.span());
        self.violations.push(Violation::new(
            self.file.clone(),
            location.line,
            location.column,
            SETTINGS_RULE,
            format!("{name} must not be retained as a legacy settings definition."),
        ));
    }
}

impl<'ast> Visit<'ast> for Visitor {
    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        self.check_style_definition(&node.ident);
        self.check_settings_definition(&node.ident);
        if STRUCTS.contains(&node.ident.unraw().to_string().as_str()) {
            for field in &node.fields {
                self.check_field(field);
            }
        }
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        self.check_style_definition(&node.ident);
        self.check_settings_definition(&node.ident);
        syn::visit::visit_item_enum(self, node);
    }

    fn visit_item_type(&mut self, node: &'ast syn::ItemType) {
        self.check_style_definition(&node.ident);
        self.check_settings_definition(&node.ident);
        syn::visit::visit_item_type(self, node);
    }
}

#[cfg(test)]
#[path = "legacy_config_generic_fields_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "legacy_settings_definition_tests.rs"]
mod settings_tests;

#[cfg(test)]
#[path = "legacy_theme_definition_tests.rs"]
mod theme_tests;
