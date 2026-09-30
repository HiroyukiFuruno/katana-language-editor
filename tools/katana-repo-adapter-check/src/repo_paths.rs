use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::config::KatanaRepoAdapterCheckConfig;

pub(crate) struct KatanaRepoAdapterPaths {
    root: PathBuf,
    manifest: PathBuf,
    wrapper: PathBuf,
    adapter: PathBuf,
    compose_linux: PathBuf,
    compose_windows: PathBuf,
    workflow_test_and_build: PathBuf,
    workflow_release_readiness: PathBuf,
    workflow_build_and_release: PathBuf,
}

impl KatanaRepoAdapterPaths {
    pub(crate) fn new(config: KatanaRepoAdapterCheckConfig) -> Self {
        let root = config.repo_root;
        Self {
            compose_linux: root.join("platforms/linux/ci/compose.yml"),
            compose_windows: root.join("platforms/windows/ci/compose.yml"),
            manifest: root.join("crates/katana-ui/Cargo.toml"),
            wrapper: root.join("crates/katana-ui/tests/ui_integration_parallel.rs"),
            adapter: root
                .join("crates/katana-ui/tests/integration/editor/kle_downstream_adapter.rs"),
            workflow_test_and_build: root.join(".github/workflows/test-and-build.yml"),
            workflow_release_readiness: root.join(".github/workflows/release-readiness.yml"),
            workflow_build_and_release: root.join(".github/workflows/build-and-release.yml"),
            root,
        }
    }

    pub(crate) fn validate_root(&self) -> Result<(), String> {
        let root = self.root.display().to_string();
        if root.contains("katana-language-editor/tmp") || root.contains("katana-kle-adapter-verify")
        {
            return Err(format!(
                "scratch KatanA clone is not valid release evidence: {root}"
            ));
        }
        if !self.root.is_dir() {
            return Err(format!("KatanA repo root does not exist: {root}"));
        }
        Ok(())
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    pub(crate) fn manifest_path(&self) -> &Path {
        &self.manifest
    }

    pub(crate) fn wrapper_path(&self) -> &Path {
        &self.wrapper
    }

    pub(crate) fn adapter_path(&self) -> &Path {
        &self.adapter
    }

    pub(crate) fn compose_linux_path(&self) -> &Path {
        &self.compose_linux
    }

    pub(crate) fn compose_windows_path(&self) -> &Path {
        &self.compose_windows
    }

    pub(crate) fn workflow_test_and_build_path(&self) -> &Path {
        &self.workflow_test_and_build
    }

    pub(crate) fn workflow_release_readiness_path(&self) -> &Path {
        &self.workflow_release_readiness
    }

    pub(crate) fn workflow_build_and_release_path(&self) -> &Path {
        &self.workflow_build_and_release
    }

    pub(crate) fn read_optional(&self, path: &Path) -> Result<Option<String>, String> {
        match fs::read_to_string(path) {
            Ok(contents) => Ok(Some(contents)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(format!("failed to read {}: {error}", path.display())),
        }
    }
}
