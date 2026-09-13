use crate::physical_bootstrap_types::{LaunchRequest, WorkspaceConfigError};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

const WORKSPACE_CONFIG_FILE_NAME: &str = "workspace.json";

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
struct GlobalWorkspaceState {
    #[serde(default)]
    persisted: Vec<String>,
    #[serde(default)]
    histories: Vec<String>,
    #[serde(default)]
    open_workspace_tabs: Vec<String>,
    #[serde(default)]
    active_workspace: Option<String>,
}

impl fmt::Display for WorkspaceConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Serialization => "workspace config serialization failed",
            Self::Write => "workspace config write failed",
            Self::Verification => "workspace config verification failed",
        })
    }
}

impl std::error::Error for WorkspaceConfigError {}

impl LaunchRequest {
    pub(crate) fn prepare_workspace_config(&self) -> Result<(), WorkspaceConfigError> {
        let workspace = self
            .workspace_fixture
            .to_str()
            .ok_or(WorkspaceConfigError::Serialization)?
            .to_owned();
        let state = GlobalWorkspaceState {
            persisted: vec![workspace.clone()],
            histories: vec![workspace.clone()],
            open_workspace_tabs: vec![workspace.clone()],
            active_workspace: Some(workspace),
        };
        let contents = serde_json::to_string_pretty(&state)
            .map_err(|_| WorkspaceConfigError::Serialization)?;
        let path = self.config_dir.join(WORKSPACE_CONFIG_FILE_NAME);
        std::fs::write(&path, contents).map_err(|_| WorkspaceConfigError::Write)?;
        verify_workspace_config(&path, &state)
    }

    #[cfg(test)]
    pub(crate) fn workspace_config_path(&self) -> std::path::PathBuf {
        self.config_dir.join(WORKSPACE_CONFIG_FILE_NAME)
    }
}

fn verify_workspace_config(
    path: &Path,
    expected: &GlobalWorkspaceState,
) -> Result<(), WorkspaceConfigError> {
    let contents = std::fs::read_to_string(path).map_err(|_| WorkspaceConfigError::Verification)?;
    let actual: serde_json::Value =
        serde_json::from_str(&contents).map_err(|_| WorkspaceConfigError::Verification)?;
    let expected =
        serde_json::to_value(expected).map_err(|_| WorkspaceConfigError::Serialization)?;
    if actual == expected {
        Ok(())
    } else {
        Err(WorkspaceConfigError::Verification)
    }
}

#[cfg(test)]
mod tests {
    use super::{GlobalWorkspaceState, verify_workspace_config};
    use crate::physical_bootstrap_types::WorkspaceConfigError;
    use std::fs;

    #[test]
    fn workspace_config_verification_failure_is_typed() {
        let root = std::env::temp_dir().join(format!(
            "katana-host-e2e-workspace-config-test-{}",
            std::process::id()
        ));
        assert!(fs::create_dir_all(&root).is_ok());
        let path = root.join("workspace.json");
        let state = GlobalWorkspaceState {
            persisted: vec!["/workspace".to_owned()],
            histories: vec!["/workspace".to_owned()],
            open_workspace_tabs: vec!["/workspace".to_owned()],
            active_workspace: Some("/workspace".to_owned()),
        };
        assert!(fs::write(&path, "{}").is_ok());

        assert_eq!(
            verify_workspace_config(&path, &state),
            Err(WorkspaceConfigError::Verification)
        );
        let _ = fs::remove_dir_all(root);
    }
}
