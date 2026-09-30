use std::fs;
use std::path::{Path, PathBuf};

use super::super::context_menu_manifest::ContextMenuTargetManifest;
use super::super::model::{
    ACTION_ORIGINS_ARTIFACT, ActionOriginsArtifact, BRANCH_CATALOG_ARTIFACT, BranchCatalogArtifact,
    CONTEXT_MENU_TARGET_MANIFEST_ARTIFACT, LEAF_MANIFEST_ARTIFACT, LeafManifestArtifact,
    SOURCE_CLOSURE_ARTIFACT, SOURCE_DERIVED_NATIVE_TARGET_ARTIFACT, SourceClosureArtifact,
    SourceDerivedNativeTargetRecord,
};

pub(crate) struct PublicationPaths<'a> {
    pub(crate) source_temp: &'a Path,
    pub(crate) branch_temp: &'a Path,
    pub(crate) action_temp: &'a Path,
    pub(crate) leaf_temp: &'a Path,
    pub(crate) native_target_temp: &'a Path,
    pub(crate) source_output: &'a Path,
    pub(crate) branch_output: &'a Path,
    pub(crate) action_output: &'a Path,
    pub(crate) leaf_output: &'a Path,
    pub(crate) native_target_output: &'a Path,
    pub(crate) context_menu_temp: &'a Path,
    pub(crate) context_menu_output: &'a Path,
}

pub(super) fn write_artifacts(
    artifact_dir: &Path,
    source_closure: &SourceClosureArtifact,
    branch_catalog: &BranchCatalogArtifact,
    action_origins: &ActionOriginsArtifact,
    leaf_manifest: &LeafManifestArtifact,
    native_target: &SourceDerivedNativeTargetRecord,
    context_menu: &ContextMenuTargetManifest,
) -> Result<PathBuf, String> {
    fs::create_dir_all(artifact_dir)
        .map_err(|error| format!("failed to create source-closure artifact directory: {error}"))?;
    let source_output = artifact_dir.join(SOURCE_CLOSURE_ARTIFACT);
    let branch_output = artifact_dir.join(BRANCH_CATALOG_ARTIFACT);
    let action_output = artifact_dir.join(ACTION_ORIGINS_ARTIFACT);
    let leaf_output = artifact_dir.join(LEAF_MANIFEST_ARTIFACT);
    let native_target_output = artifact_dir.join(SOURCE_DERIVED_NATIVE_TARGET_ARTIFACT);
    let context_menu_output = artifact_dir.join(CONTEXT_MENU_TARGET_MANIFEST_ARTIFACT);
    for output in [
        &source_output,
        &branch_output,
        &action_output,
        &leaf_output,
        &native_target_output,
        &context_menu_output,
    ] {
        if output.exists() {
            return Err(format!(
                "refusing to overwrite immutable source-closure artifact: {}",
                output.display()
            ));
        }
    }

    let source_temp = artifact_dir.join(format!(
        ".{}.{}.tmp",
        SOURCE_CLOSURE_ARTIFACT,
        std::process::id()
    ));
    let branch_temp = artifact_dir.join(format!(
        ".{}.{}.tmp",
        BRANCH_CATALOG_ARTIFACT,
        std::process::id()
    ));
    let action_temp = artifact_dir.join(format!(
        ".{}.{}.tmp",
        ACTION_ORIGINS_ARTIFACT,
        std::process::id()
    ));
    let leaf_temp = artifact_dir.join(format!(
        ".{}.{}.tmp",
        LEAF_MANIFEST_ARTIFACT,
        std::process::id()
    ));
    let native_target_temp = artifact_dir.join(format!(
        ".{}.{}.tmp",
        SOURCE_DERIVED_NATIVE_TARGET_ARTIFACT,
        std::process::id()
    ));
    let context_menu_temp = artifact_dir.join(format!(
        ".{}.{}.tmp",
        CONTEXT_MENU_TARGET_MANIFEST_ARTIFACT,
        std::process::id()
    ));
    for output in [
        &source_temp,
        &branch_temp,
        &action_temp,
        &leaf_temp,
        &native_target_temp,
        &context_menu_temp,
    ] {
        if output.exists() {
            return Err(format!(
                "refusing to overwrite immutable source-closure artifact: {}",
                output.display()
            ));
        }
    }

    let mut source_bytes = serde_json::to_vec_pretty(source_closure)
        .map_err(|error| format!("failed to write source-closure artifact: {error}"))?;
    source_bytes.push(b'\n');
    let mut branch_bytes = serde_json::to_vec_pretty(branch_catalog)
        .map_err(|error| format!("failed to write branch-catalog artifact: {error}"))?;
    branch_bytes.push(b'\n');
    let mut action_bytes = serde_json::to_vec_pretty(action_origins)
        .map_err(|error| format!("failed to write action-origins artifact: {error}"))?;
    action_bytes.push(b'\n');
    let mut leaf_bytes = serde_json::to_vec_pretty(leaf_manifest)
        .map_err(|error| format!("failed to write leaf manifest: {error}"))?;
    leaf_bytes.push(b'\n');
    let mut native_target_bytes = serde_json::to_vec_pretty(native_target).map_err(|error| {
        format!("failed to write source-derived native target artifact: {error}")
    })?;
    native_target_bytes.push(b'\n');
    fs::write(&source_temp, source_bytes)
        .map_err(|error| format!("failed to finalize source-closure artifact: {error}"))?;
    fs::write(&branch_temp, branch_bytes)
        .map_err(|error| format!("failed to finalize branch-catalog artifact: {error}"))?;
    fs::write(&action_temp, action_bytes)
        .map_err(|error| format!("failed to finalize action-origins artifact: {error}"))?;
    fs::write(&leaf_temp, leaf_bytes)
        .map_err(|error| format!("failed to finalize leaf manifest artifact: {error}"))?;
    fs::write(&native_target_temp, native_target_bytes).map_err(|error| {
        format!("failed to finalize source-derived native target artifact: {error}")
    })?;
    let mut context_menu_bytes = serde_json::to_vec_pretty(context_menu)
        .map_err(|error| format!("failed to write context-menu target manifest: {error}"))?;
    context_menu_bytes.push(b'\n');
    fs::write(&context_menu_temp, context_menu_bytes)
        .map_err(|error| format!("failed to finalize context-menu target manifest: {error}"))?;
    publish_prepared_artifacts(&PublicationPaths {
        source_temp: &source_temp,
        branch_temp: &branch_temp,
        action_temp: &action_temp,
        leaf_temp: &leaf_temp,
        native_target_temp: &native_target_temp,
        source_output: &source_output,
        branch_output: &branch_output,
        action_output: &action_output,
        leaf_output: &leaf_output,
        native_target_output: &native_target_output,
        context_menu_temp: &context_menu_temp,
        context_menu_output: &context_menu_output,
    })?;
    Ok(source_output)
}

pub(super) fn publish_prepared_artifacts(paths: &PublicationPaths<'_>) -> Result<(), String> {
    let pairs = [
        (paths.source_temp, paths.source_output, "source-closure"),
        (paths.branch_temp, paths.branch_output, "branch-catalog"),
        (paths.action_temp, paths.action_output, "action-origins"),
        (paths.leaf_temp, paths.leaf_output, "leaf-manifest"),
        (
            paths.native_target_temp,
            paths.native_target_output,
            "source-derived-native-target",
        ),
        (
            paths.context_menu_temp,
            paths.context_menu_output,
            "context-menu-target-manifest",
        ),
    ];
    let mut published: Vec<(&str, &Path)> = Vec::new();
    for (temporary, output, name) in pairs {
        if let Err(error) = fs::rename(temporary, output) {
            let rollback_failures = published
                .iter()
                .filter_map(|(_, published_path)| {
                    fs::remove_file(published_path).err().map(|rollback_error| {
                        format!("{}: {rollback_error}", published_path.display())
                    })
                })
                .collect::<Vec<_>>();
            if rollback_failures.is_empty() {
                return Err(format!(
                    "failed to finalize {name} artifact: {error}; rolled back {} published artifacts",
                    published.len()
                ));
            }
            return Err(format!(
                "failed to finalize {name} artifact: {error}; rollback failed: {}",
                rollback_failures.join("; ")
            ));
        }
        published.push((name, output));
    }
    Ok(())
}
