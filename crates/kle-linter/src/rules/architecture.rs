use crate::diagnostics::{KleLintError, Violation};
use crate::workspace::WorkspaceModel;

use super::egui_duplication::EguiDuplicationRule;
use super::manifest_boundary::ManifestBoundaryRule;

pub const LIB_CRATE: &str = "crates/katana-language-editor";
pub const EGUI_CRATE: &str = "crates/katana-language-editor-egui";

pub struct ArchitectureRule;

impl ArchitectureRule {
    pub fn check(workspace: &WorkspaceModel) -> Result<Vec<Violation>, KleLintError> {
        let mut violations = Vec::new();
        violations.extend(ManifestBoundaryRule::check(workspace.root())?);
        violations.extend(EguiDuplicationRule::check(workspace));
        Ok(violations)
    }
}
