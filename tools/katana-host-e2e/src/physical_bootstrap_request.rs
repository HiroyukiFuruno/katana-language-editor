use crate::physical_bootstrap_types::{LaunchRequest, RequestValidationError};
use std::path::{Path, PathBuf};

pub(crate) const TARGET_DIR_NAME: &str = "target";
pub(crate) const CONFIG_DIR_NAME: &str = "config";
impl std::fmt::Display for RequestValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyPath(name) => write!(formatter, "{name} path is empty"),
            Self::PathNotFound(name) => write!(formatter, "{name} path does not exist"),
            Self::NotDirectory(name) => write!(formatter, "{name} path is not a directory"),
            Self::NotFile(name) => write!(formatter, "{name} path is not a file"),
            Self::MissingCargoManifest => {
                formatter.write_str("fixed KatanA root has no Cargo.toml")
            }
            Self::MarkdownFixtureMissing => {
                formatter.write_str("workspace fixture has no Markdown file")
            }
            Self::Io(name) => write!(formatter, "I/O failed while preparing {name}"),
            Self::FixtureCreationFailed => {
                formatter.write_str("cannot create physical test fixture")
            }
            Self::WorkspacePathNotAbsolute => {
                formatter.write_str("workspace fixture path must be absolute")
            }
            Self::WorkspacePathTraversal => {
                formatter.write_str("workspace fixture path contains traversal")
            }
        }
    }
}

impl std::error::Error for RequestValidationError {}

impl LaunchRequest {
    pub(crate) fn new(
        fixed_katana_root: impl AsRef<Path>,
        sandbox_root: impl AsRef<Path>,
        workspace_fixture: impl AsRef<Path>,
    ) -> Result<Self, RequestValidationError> {
        let fixed_katana_root =
            existing_absolute_directory(fixed_katana_root.as_ref(), "fixed_katana_root")?;
        if !fixed_katana_root.join("Cargo.toml").is_file() {
            return Err(RequestValidationError::MissingCargoManifest);
        }
        let execution_sandbox = prepare_absolute_directory(sandbox_root.as_ref(), "sandbox_root")?;
        let workspace_fixture = workspace_fixture.as_ref();
        if !workspace_fixture.is_absolute() {
            return Err(RequestValidationError::WorkspacePathNotAbsolute);
        }
        if workspace_fixture
            .components()
            .any(|component| component == std::path::Component::ParentDir)
        {
            return Err(RequestValidationError::WorkspacePathTraversal);
        }
        let workspace_fixture =
            existing_absolute_directory(workspace_fixture, "workspace_fixture")?;
        if !contains_markdown_file(&workspace_fixture) {
            return Err(RequestValidationError::MarkdownFixtureMissing);
        }
        let target_dir =
            prepare_absolute_directory(&execution_sandbox.join(TARGET_DIR_NAME), "target_dir")?;
        let config_dir =
            prepare_absolute_directory(&execution_sandbox.join(CONFIG_DIR_NAME), "config_dir")?;
        Ok(Self {
            fixed_katana_root,
            execution_sandbox,
            workspace_fixture,
            target_dir,
            config_dir,
        })
    }

    pub(crate) fn fixed_katana_root(&self) -> &Path {
        &self.fixed_katana_root
    }

    #[cfg(test)]
    pub(crate) fn execution_sandbox(&self) -> &Path {
        &self.execution_sandbox
    }

    pub fn workspace_fixture(&self) -> &Path {
        &self.workspace_fixture
    }
}

fn existing_absolute_directory(
    path: &Path,
    name: &'static str,
) -> Result<PathBuf, RequestValidationError> {
    if path.as_os_str().is_empty() {
        return Err(RequestValidationError::EmptyPath(name));
    }
    let absolute = absolute_path(path, name)?;
    let metadata =
        std::fs::metadata(&absolute).map_err(|_| RequestValidationError::PathNotFound(name))?;
    if !metadata.is_dir() {
        return Err(RequestValidationError::NotDirectory(name));
    }
    std::fs::canonicalize(absolute).map_err(|_| RequestValidationError::Io(name))
}

fn prepare_absolute_directory(
    path: &Path,
    name: &'static str,
) -> Result<PathBuf, RequestValidationError> {
    if path.as_os_str().is_empty() {
        return Err(RequestValidationError::EmptyPath(name));
    }
    let absolute = absolute_path(path, name)?;
    std::fs::create_dir_all(&absolute).map_err(|_| RequestValidationError::Io(name))?;
    let metadata =
        std::fs::metadata(&absolute).map_err(|_| RequestValidationError::PathNotFound(name))?;
    if !metadata.is_dir() {
        return Err(RequestValidationError::NotDirectory(name));
    }
    std::fs::canonicalize(absolute).map_err(|_| RequestValidationError::Io(name))
}

fn absolute_path(path: &Path, name: &'static str) -> Result<PathBuf, RequestValidationError> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    std::env::current_dir()
        .map(|current| current.join(path))
        .map_err(|_| RequestValidationError::Io(name))
}

fn contains_markdown_file(root: &Path) -> bool {
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let entries = match std::fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(_) => return false,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            if path.extension().is_some_and(|extension| extension == "md") {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{CONFIG_DIR_NAME, TARGET_DIR_NAME};
    use crate::physical_bootstrap_types::WorkspaceConfigError;
    use crate::{LaunchRequest, RequestValidationError};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEST_ID: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        fixed: PathBuf,
        sandbox: PathBuf,
        workspace: PathBuf,
    }

    impl Fixture {
        fn new(with_markdown: bool) -> Self {
            let id = NEXT_TEST_ID.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!("katana-host-e2e-{id}"));
            let fixed = root.join("fixed");
            let sandbox = root.join("sandbox");
            let workspace = root.join("workspace");
            fs::create_dir_all(&fixed).expect("fixed root");
            fs::create_dir_all(&workspace).expect("workspace");
            fs::write(fixed.join("Cargo.toml"), "[workspace]\n").expect("manifest");
            if with_markdown {
                fs::write(workspace.join("fixture.md"), "# fixture\n").expect("markdown");
            }
            Self {
                root,
                fixed,
                sandbox,
                workspace,
            }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn request_creates_isolated_sandbox_and_accepts_markdown_fixture() {
        let fixture = Fixture::new(true);
        let request = LaunchRequest::new(&fixture.fixed, &fixture.sandbox, &fixture.workspace)
            .expect("valid request");
        assert!(request.execution_sandbox().is_absolute());
        assert!(request.execution_sandbox().join(TARGET_DIR_NAME).is_dir());
        assert!(request.execution_sandbox().join(CONFIG_DIR_NAME).is_dir());
        assert!(request.workspace_fixture().join("fixture.md").is_file());
    }

    #[test]
    fn invalid_workspace_without_markdown_fails_closed() {
        let fixture = Fixture::new(false);
        assert_eq!(
            LaunchRequest::new(&fixture.fixed, &fixture.sandbox, &fixture.workspace),
            Err(RequestValidationError::MarkdownFixtureMissing)
        );
    }

    #[test]
    fn nonexistent_workspace_fails_closed() {
        let fixture = Fixture::new(true);
        let missing = fixture.root.join("missing-workspace");
        assert_eq!(
            LaunchRequest::new(&fixture.fixed, &fixture.sandbox, missing),
            Err(RequestValidationError::PathNotFound("workspace_fixture"))
        );
    }

    #[test]
    fn workspace_path_traversal_is_rejected_before_filesystem_observation() {
        let fixture = Fixture::new(true);
        let traversal = fixture.workspace.join("..").join("workspace");
        assert_eq!(
            LaunchRequest::new(&fixture.fixed, &fixture.sandbox, traversal),
            Err(RequestValidationError::WorkspacePathTraversal)
        );
    }

    #[test]
    fn workspace_config_uses_the_canonical_workspace_identity_for_every_state_field() {
        let fixture = Fixture::new(true);
        let request = LaunchRequest::new(&fixture.fixed, &fixture.sandbox, &fixture.workspace)
            .expect("valid request");

        request
            .prepare_workspace_config()
            .expect("workspace config");
        let contents = fs::read_to_string(request.workspace_config_path()).expect("config");
        let value: serde_json::Value = serde_json::from_str(&contents).expect("JSON");
        let workspace = request.workspace_fixture().to_str().expect("UTF-8 path");
        assert_eq!(value["persisted"], serde_json::json!([workspace]));
        assert_eq!(value["histories"], serde_json::json!([workspace]));
        assert_eq!(value["open_workspace_tabs"], serde_json::json!([workspace]));
        assert_eq!(value["active_workspace"], serde_json::json!(workspace));
    }

    #[test]
    fn workspace_config_write_failure_is_typed_and_fail_closed() {
        let fixture = Fixture::new(true);
        let mut request = LaunchRequest::new(&fixture.fixed, &fixture.sandbox, &fixture.workspace)
            .expect("valid request");
        fs::write(request.config_dir.join("config-file"), "not a directory").expect("fixture");
        request.config_dir = request.config_dir.join("config-file");

        assert_eq!(
            request.prepare_workspace_config(),
            Err(WorkspaceConfigError::Write)
        );
    }
}
