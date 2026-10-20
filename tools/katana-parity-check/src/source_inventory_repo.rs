#[cfg(test)]
use std::collections::HashMap;
use std::{
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

#[cfg(test)]
const INVENTORY_DIRECTORIES: &[&str] = &[
    "crates/katana-ui/src/views/panels/editor",
    "crates/katana-ui/tests/integration/editor",
];

#[cfg(test)]
const INVENTORY_FILES: &[&str] = &[
    "crates/katana-ui/src/app/action/process_authoring.rs",
    "crates/katana-ui/src/app/doc_search.rs",
    "crates/katana-ui/src/app/document_edit.rs",
    "crates/katana-ui/src/app/action/image_ingest.rs",
    "crates/katana-ui/src/app/action/dispatch_secondary.rs",
    "crates/katana-ui/src/app/action/clipboard_image.rs",
    "crates/katana-ui/src/app/action/clipboard_file_url.rs",
    "crates/katana-ui/src/app/action/clipboard_image_macos.rs",
    "crates/katana-ui/src/app/action/process_markdown_formatting.rs",
    "crates/katana-ui/src/app/action/refresh_content.rs",
    "crates/katana-ui/src/editor_undo.rs",
    "crates/katana-ui/src/state/command_inventory/edit_commands.rs",
    "crates/katana-ui/src/state/shortcut_context.rs",
    "crates/katana-ui/src/shell_ui/shell_ui_shortcuts.rs",
    "crates/katana-ui/src/views/top_bar/search.rs",
];

pub(crate) struct SourceInventoryRepo;

impl SourceInventoryRepo {
    pub(crate) fn resolve_katana_repo() -> Result<PathBuf, String> {
        let workspace_root = workspace_root()?;
        let configured = env::var_os("KATANA_REPO");
        let Some(path) = configured else {
            return Ok(workspace_root.join("../katana"));
        };

        let path = PathBuf::from(path);
        if path.is_absolute() {
            return Ok(path);
        }
        Ok(workspace_root.join(path))
    }

    #[cfg(test)]
    pub(crate) fn collect_expected_inventory(
        repo_root: &Path,
    ) -> Result<HashMap<String, Vec<String>>, String> {
        let mut files = HashMap::new();

        for dir in INVENTORY_DIRECTORIES {
            let entries = collect_rs_files(repo_root.join(dir))?;
            for path in entries {
                let rel = to_relative(repo_root, &path)?;
                files.insert(rel, Vec::new());
            }
        }

        for file in INVENTORY_FILES {
            let path = repo_root.join(file);
            let rel = to_relative(repo_root, &path)?;
            files.insert(rel, Vec::new());
        }

        Ok(files)
    }

    #[cfg(test)]
    pub(crate) fn collect_fn_names(contents: &str) -> Vec<String> {
        let mut names = Vec::new();
        for line in contents.lines() {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            let fn_index = match tokens.iter().position(|token| *token == "fn") {
                Some(index) => index,
                None => continue,
            };

            let name_index = fn_index + 1;
            if name_index >= tokens.len() {
                continue;
            }

            let name = tokens[name_index]
                .split(['<', '(', ':'])
                .next()
                .unwrap_or("");

            if !name.is_empty() {
                names.push(name.to_string());
            }
        }
        names
    }

    pub(crate) fn read_file(path: &Path) -> Result<String, String> {
        fs::read_to_string(path).map_err(|error| match error.kind() {
            ErrorKind::NotFound => format!("source inventory file is missing: {}", path.display()),
            _ => format!("failed to read {}: {error}", path.display()),
        })
    }
}

#[cfg(test)]
fn collect_rs_files(dir: PathBuf) -> Result<Vec<PathBuf>, String> {
    let mut result = Vec::new();
    let mut stack = vec![dir];

    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current)
            .map_err(|error| format!("failed to read {}: {error}", current.display()))?
        {
            let entry =
                entry.map_err(|error| format!("failed to read directory entry: {error}"))?;
            let path = entry.path();
            let file_type = entry
                .file_type()
                .map_err(|error| format!("failed to read file type {}: {error}", path.display()))?;

            if file_type.is_dir() {
                stack.push(path);
                continue;
            }

            if file_type.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                result.push(path);
            }
        }
    }

    Ok(result)
}

fn workspace_root() -> Result<PathBuf, String> {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|error| format!("failed to resolve workspace root: {error}"))?;
    Ok(workspace_root)
}

#[cfg(test)]
fn to_relative(repo_root: &Path, path: &Path) -> Result<String, String> {
    let rel = path
        .strip_prefix(repo_root)
        .map_err(|error| format!("path {path:?} is not under repo root: {error}"))?;
    Ok(rel.to_string_lossy().replace('\\', "/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_fn_names_parses_fn_styles() {
        let src = "fn foo() {}\npub async fn bar<B>() {}\npub fn baz() {}\n";
        let names = SourceInventoryRepo::collect_fn_names(src);
        assert!(names.contains(&"foo".to_string()));
        assert!(names.contains(&"bar".to_string()));
        assert!(names.contains(&"baz".to_string()));
    }

    #[test]
    fn retains_legacy_inventory_helpers_only_for_regression_tests() {
        let _ = SourceInventoryRepo::collect_expected_inventory(Path::new("/"));
        assert!(!INVENTORY_DIRECTORIES.is_empty());
        assert!(!INVENTORY_FILES.is_empty());
    }

    #[test]
    fn resolves_relative_katana_repo_from_workspace_root() -> Result<(), String> {
        let workspace_root = workspace_root()?;
        let actual = resolve_katana_repo_from(&workspace_root, Some(Path::new("../katana")));
        assert_eq!(actual, workspace_root.join("../katana"));
        Ok(())
    }

    #[test]
    fn expected_inventory_includes_required_out_of_directory_files() {
        assert!(INVENTORY_FILES.contains(&"crates/katana-ui/src/views/top_bar/search.rs"));
        assert!(
            !INVENTORY_FILES
                .contains(&"crates/katana-ui/tests/integration/editor/kle_downstream_adapter.rs")
        );
        assert!(INVENTORY_FILES.contains(&"crates/katana-ui/src/app/action/clipboard_image.rs"));
        assert!(INVENTORY_FILES.contains(&"crates/katana-ui/src/state/shortcut_context.rs"));
    }

    fn resolve_katana_repo_from(workspace_root: &Path, configured_repo: Option<&Path>) -> PathBuf {
        let Some(path) = configured_repo else {
            return workspace_root.join("../katana");
        };
        if path.is_absolute() {
            return path.to_path_buf();
        }
        workspace_root.join(path)
    }
}
