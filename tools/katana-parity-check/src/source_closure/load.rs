use std::path::Path;

use serde::de::DeserializeOwned;

use super::artifact_paths::ArtifactPaths;
use super::context_menu_manifest::ContextMenuTargetManifest;
use super::model::{
    ActionOriginsArtifact, BranchCatalogArtifact, ExecutionRecordArtifact, LeafManifestArtifact,
    SourceClosureArtifact, StorybookArtifacts,
};

pub(super) struct LoadedArtifacts {
    pub(super) source_closure: SourceClosureArtifact,
    pub(super) action_origins: ActionOriginsArtifact,
    pub(super) branch_catalog: BranchCatalogArtifact,
    pub(super) leaf_manifest: LeafManifestArtifact,
    pub(super) execution_record: ExecutionRecordArtifact,
    pub(super) storybook: StorybookArtifacts,
    pub(super) context_menu_target_manifest: Option<ContextMenuTargetManifest>,
}

impl LoadedArtifacts {
    pub(super) fn from_paths(artifacts: &ArtifactPaths) -> Result<Self, String> {
        Ok(Self {
            source_closure: read_artifact(&artifacts.source_closure)
                .map_err(|error| format!("source-closure: {error}"))?,
            action_origins: read_artifact(&artifacts.action_origins)
                .map_err(|error| format!("action-origins: {error}"))?,
            branch_catalog: read_artifact(&artifacts.branch_catalog)
                .map_err(|error| format!("branch-catalog: {error}"))?,
            leaf_manifest: read_artifact(&artifacts.leaf_manifest)
                .map_err(|error| format!("leaf-manifest: {error}"))?,
            execution_record: read_artifact(&artifacts.execution_record)
                .map_err(|error| format!("execution-record: {error}"))?,
            storybook: read_artifact(&artifacts.storybook_artifacts)
                .map_err(|error| format!("storybook-artifacts: {error}"))?,
            context_menu_target_manifest: if artifacts.context_menu_target_manifest.exists() {
                Some(
                    read_artifact(&artifacts.context_menu_target_manifest)
                        .map_err(|error| format!("context-menu-target-manifest: {error}"))?,
                )
            } else {
                None
            },
        })
    }
}

pub(super) fn read_artifact<T: DeserializeOwned>(path: &Path) -> Result<T, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|error| format!("missing or unreadable file {}: {error}", path.display()))?;
    serde_json::from_str(&content)
        .map_err(|error| format!("invalid JSON in {}: {error}", path.display()))
}
