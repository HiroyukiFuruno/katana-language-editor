use std::{
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use crate::actual_repo_adapter_evidence::{
    FORBIDDEN_PATCH_EVIDENCE, REQUIRED_ACTUAL_ADAPTER_EVIDENCE, REQUIRED_ACTUAL_BASELINE_TESTS,
    REQUIRED_ACTUAL_MANIFEST_EVIDENCE, REQUIRED_ACTUAL_TOOLBAR_POPUP_TESTS,
    REQUIRED_ACTUAL_WRAPPER_EVIDENCE,
};
use crate::actual_repo_tasks::ActualRepoTaskStateAudit;

pub(crate) struct ActualRepoAdapterEvidence;

impl ActualRepoAdapterEvidence {
    pub(crate) fn validate_task_state_matches_actual_repo(
        tasks_document: &str,
        matrix_has_open_gaps: bool,
    ) -> Result<(), String> {
        let actual_evidence_present = actual_repo_evidence_present()?;
        ActualRepoTaskStateAudit::validate_task_state_matches_evidence(
            tasks_document,
            actual_evidence_present,
            matrix_has_open_gaps,
        )
    }
}

fn actual_repo_evidence_present() -> Result<bool, String> {
    actual_repo_evidence_present_at(&actual_repo_root()?)
}

fn actual_repo_root() -> Result<PathBuf, String> {
    let workspace_root = workspace_root()?;
    Ok(resolve_katana_repo(
        &workspace_root,
        env::var_os("KATANA_REPO").as_deref().map(Path::new),
    ))
}

fn workspace_root() -> Result<PathBuf, String> {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|error| format!("failed to resolve workspace root: {error}"))?;
    Ok(workspace_root)
}

fn resolve_katana_repo(workspace_root: &Path, configured_repo: Option<&Path>) -> PathBuf {
    let Some(path) = configured_repo else {
        return workspace_root.join("../katana");
    };

    let path = PathBuf::from(path);
    if path.is_absolute() {
        return path;
    }
    workspace_root.join(path)
}

fn actual_repo_evidence_present_at(repo_root: &Path) -> Result<bool, String> {
    let Some(manifest) = read_optional(repo_root.join("crates/katana-ui/Cargo.toml"))? else {
        return Ok(false);
    };
    if !contains_all(&manifest, REQUIRED_ACTUAL_MANIFEST_EVIDENCE) {
        return Ok(false);
    }

    let Some(wrapper) =
        read_optional(repo_root.join("crates/katana-ui/tests/ui_integration_parallel.rs"))?
    else {
        return Ok(false);
    };
    if !contains_all(&wrapper, REQUIRED_ACTUAL_WRAPPER_EVIDENCE) {
        return Ok(false);
    }

    let Some(adapter) = read_optional(
        repo_root.join("crates/katana-ui/tests/integration/editor/kle_downstream_adapter.rs"),
    )?
    else {
        return Ok(false);
    };
    if !contains_all(&adapter, REQUIRED_ACTUAL_ADAPTER_EVIDENCE) {
        return Ok(false);
    }
    if FORBIDDEN_PATCH_EVIDENCE
        .iter()
        .any(|needle| adapter.contains(needle))
    {
        return Ok(false);
    }

    let Some(ui_tests) =
        read_optional(repo_root.join("crates/katana-ui/tests/integration/editor/ui.rs"))?
    else {
        return Ok(false);
    };
    if !contains_all(&ui_tests, REQUIRED_ACTUAL_BASELINE_TESTS) {
        return Ok(false);
    }

    let Some(toolbar_popup_tests) =
        read_optional(repo_root.join("crates/katana-ui/src/views/panels/editor/toolbar_popup.rs"))?
    else {
        return Ok(false);
    };
    if !contains_all(&toolbar_popup_tests, REQUIRED_ACTUAL_TOOLBAR_POPUP_TESTS) {
        return Ok(false);
    }

    Ok(true)
}

fn read_optional(path: PathBuf) -> Result<Option<String>, String> {
    match fs::read_to_string(&path) {
        Ok(contents) => Ok(Some(contents)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("failed to read {}: {error}", path.display())),
    }
}

fn contains_all(document: &str, needles: &[&str]) -> bool {
    needles.iter().all(|needle| document.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn resolves_relative_katana_repo_from_workspace_root() -> Result<(), String> {
        let workspace_root = workspace_root()?;
        let actual = resolve_katana_repo(
            &workspace_root,
            Some(Path::new("../katana-kle-v0.1.0-adapter")),
        );
        let expected = workspace_root.join("../katana-kle-v0.1.0-adapter");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn preserves_absolute_katana_repo() -> Result<(), String> {
        let workspace_root = workspace_root()?;
        let absolute_repo = Path::new("/tmp/katana-parity-check-absolute-repo");
        let actual = resolve_katana_repo(&workspace_root, Some(absolute_repo));
        assert_eq!(actual, absolute_repo);
        Ok(())
    }

    #[test]
    fn default_repo_is_workspace_relative_katana() -> Result<(), String> {
        let workspace_root = workspace_root()?;
        let actual = resolve_katana_repo(&workspace_root, None);
        assert_eq!(actual, workspace_root.join("../katana"));
        Ok(())
    }
}
