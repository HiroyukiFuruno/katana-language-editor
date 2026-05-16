use crate::diagnostics::{KleLintError, Violation};
use std::path::Path;

use super::architecture::{EGUI_CRATE, FLOEM_CRATE, LIB_CRATE};
use super::manifest_reader::ManifestReader;
use super::ui_dependency_policy::UiDependencyPolicy;

const FLOEM_GIT_URL: &str = "https://github.com/lapce/floem";

pub(super) struct ManifestBoundaryRule;

impl ManifestBoundaryRule {
    pub(super) fn check(root: &Path) -> Result<Vec<Violation>, KleLintError> {
        let mut violations = Vec::new();
        let root_manifest = root.join("Cargo.toml");
        let lib_manifest = root.join(LIB_CRATE).join("Cargo.toml");
        let egui_manifest = root.join(EGUI_CRATE).join("Cargo.toml");
        let floem_manifest = root.join(FLOEM_CRATE).join("Cargo.toml");
        Self::check_workspace_floem_dependency(&root_manifest, &mut violations)?;
        Self::check_editor_manifest(&lib_manifest, &mut violations)?;
        Self::check_egui_manifest(&egui_manifest, &mut violations)?;
        Self::check_floem_manifest(&floem_manifest, &mut violations)?;
        Ok(violations)
    }

    fn check_editor_manifest(
        path: &Path,
        violations: &mut Vec<Violation>,
    ) -> Result<(), KleLintError> {
        let manifest = ManifestReader::read(path)?;
        for dependency in ManifestReader::dependency_names(&manifest) {
            if !Self::is_editor_boundary_violation(&dependency) {
                continue;
            }
            violations.push(Self::manifest_violation(path, dependency));
        }
        Ok(())
    }

    fn check_workspace_floem_dependency(
        path: &Path,
        violations: &mut Vec<Violation>,
    ) -> Result<(), KleLintError> {
        let manifest = ManifestReader::read(path)?;
        let Some(dependency) = ManifestReader::workspace_dependency(&manifest, "floem") else {
            violations.push(Self::simple_violation(
                path,
                "floem-git-dependency",
                "workspace.dependencies must define floem as a git dependency.",
            ));
            return Ok(());
        };
        if !ManifestReader::has_git_url(dependency, FLOEM_GIT_URL)
            || !ManifestReader::has_pinned_rev(dependency)
            || ManifestReader::has_version(dependency)
        {
            violations.push(Self::simple_violation(
                path,
                "floem-git-dependency",
                "floem must use https://github.com/lapce/floem with a pinned rev, not crates.io.",
            ));
        }
        Ok(())
    }

    fn check_floem_manifest(
        path: &Path,
        violations: &mut Vec<Violation>,
    ) -> Result<(), KleLintError> {
        let manifest = ManifestReader::read(path)?;
        Self::require_dependency(
            path,
            &manifest,
            "katana-language-editor",
            "floem-library-boundary",
            "floem crate must depend on the neutral API instead of owning editor contracts.",
            violations,
        );
        Self::require_floem_workspace_dependency(path, &manifest, violations);
        Ok(())
    }

    fn require_dependency(
        path: &Path,
        manifest: &toml::Value,
        dependency: &str,
        rule: &'static str,
        message: &str,
        violations: &mut Vec<Violation>,
    ) {
        if ManifestReader::dependency_value(manifest, dependency).is_some() {
            return;
        }
        violations.push(Self::simple_violation(path, rule, message));
    }

    fn require_floem_workspace_dependency(
        path: &Path,
        manifest: &toml::Value,
        violations: &mut Vec<Violation>,
    ) {
        let Some(dependency) = ManifestReader::dependency_value(manifest, "floem") else {
            violations.push(Self::simple_violation(
                path,
                "floem-git-dependency",
                "floem crate must depend on the workspace floem git dependency.",
            ));
            return;
        };
        if ManifestReader::is_workspace_dependency(dependency) {
            return;
        }
        violations.push(Self::simple_violation(
            path,
            "floem-git-dependency",
            "floem crate must inherit floem from workspace.dependencies.",
        ));
    }

    fn check_egui_manifest(
        path: &Path,
        violations: &mut Vec<Violation>,
    ) -> Result<(), KleLintError> {
        let manifest = ManifestReader::read(path)?;
        let dependencies = ManifestReader::dependency_names(&manifest);
        if dependencies.iter().any(|it| it == "katana-language-editor") {
            return Ok(());
        }
        violations.push(Violation::new(
            path.to_path_buf(),
            1,
            1,
            "egui-library-boundary",
            "egui crate must depend on the neutral API instead of owning editor contracts.",
        ));
        Ok(())
    }

    fn is_editor_boundary_violation(dependency: &str) -> bool {
        UiDependencyPolicy::is_ui_dependency(dependency)
            || dependency == "katana-language-editor-egui"
    }

    fn manifest_violation(path: &Path, dependency: String) -> Violation {
        Self::simple_violation(
            path,
            "editor-boundary",
            &format!("language editor crate must not depend on UI or egui crate `{dependency}`."),
        )
    }

    fn simple_violation(path: &Path, rule: &'static str, message: &str) -> Violation {
        Violation::new(path.to_path_buf(), 1, 1, rule, message)
    }
}
