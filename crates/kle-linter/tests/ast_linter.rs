use kle_linter::{KleLintError, KleLinter, ViolationReport};
use std::path::{Path, PathBuf};

fn workspace_root() -> Result<PathBuf, KleLintError> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let Some(crates_dir) = manifest_dir.parent() else {
        return Err(KleLintError::WorkspaceRoot {
            path: manifest_dir.to_path_buf(),
        });
    };
    let Some(root) = crates_dir.parent() else {
        return Err(KleLintError::WorkspaceRoot {
            path: crates_dir.to_path_buf(),
        });
    };
    Ok(root.to_path_buf())
}

#[test]
fn ast_linter_kal_standard_rules_are_clean() {
    katana_ast_lint::KatanaAstLint::from_workspace().assert_clean();
}

#[test]
fn ast_linter_workspace_rules() -> Result<(), KleLintError> {
    let root = workspace_root()?;
    let violations = KleLinter::lint_workspace(&root)?;
    assert!(
        violations.is_empty(),
        "{}",
        ViolationReport::format(&violations)
    );
    Ok(())
}
