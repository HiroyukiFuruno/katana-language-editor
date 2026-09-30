use std::path::{Component, Path};

use super::artifact_paths::ArtifactPaths;
use super::context_menu_manifest;
use super::execution_validation;
use super::fingerprint::sha256_hex;
use super::kle_release_gate;
use super::leaf_validation;
use super::load::LoadedArtifacts;
use super::model::StorybookArtifacts;
use super::root_validation;
use super::source_validation;
use super::storybook_media_validation::is_numbered_stage_png;

pub use super::artifact_validation_mode::ArtifactValidationMode;

pub struct SourceClosureArtifactValidator;

impl SourceClosureArtifactValidator {
    pub fn validate(artifacts: &ArtifactPaths) -> Result<(), String> {
        Self::validate_for_mode(artifacts, ArtifactValidationMode::KleRelease)
    }

    pub fn validate_for_mode(
        artifacts: &ArtifactPaths,
        mode: ArtifactValidationMode,
    ) -> Result<(), String> {
        let loaded = LoadedArtifacts::from_paths(artifacts)?;
        let mut errors = Vec::new();
        root_validation::validate_root_consistency(
            [
                ("source-closure", &loaded.source_closure.root),
                ("action-origins", &loaded.action_origins.root),
                ("branch-catalog", &loaded.branch_catalog.root),
                ("leaf-manifest", &loaded.leaf_manifest.root),
                ("execution-record", &loaded.execution_record.root),
                ("storybook-artifacts", &loaded.storybook.root),
            ],
            &mut errors,
        );
        root_validation::reject_static_leaf_claims(
            [
                ("source-closure", &loaded.source_closure.root),
                ("action-origins", &loaded.action_origins.root),
                ("branch-catalog", &loaded.branch_catalog.root),
                ("leaf-manifest", &loaded.leaf_manifest.root),
                ("execution-record", &loaded.execution_record.root),
                ("storybook-artifacts", &loaded.storybook.root),
            ],
            &mut errors,
        );
        source_validation::validate(&loaded.source_closure, &mut errors);
        if let Err(error) = loaded
            .leaf_manifest
            .canonical_binding
            .validate(&loaded.source_closure, &loaded.branch_catalog)
        {
            errors.push(format!("leaf-manifest canonical binding: {error}"));
        }
        if let Err(error) = loaded
            .leaf_manifest
            .canonical_binding
            .validate_projection(&loaded.branch_catalog, &loaded.leaf_manifest.leafs)
        {
            errors.push(format!("leaf-manifest projection: {error}"));
        }
        if mode != ArtifactValidationMode::Rc {
            errors.extend(
                loaded
                    .leaf_manifest
                    .canonical_binding
                    .release_completeness_errors(),
            );
        }
        match (
            loaded.source_closure.context_menu_target_manifest.is_some(),
            loaded.context_menu_target_manifest.is_some(),
        ) {
            (true, false) => errors.push(
                "source-closure declares context-menu target manifest but the separate artifact is missing"
                    .to_string(),
            ),
            (false, true) => errors.push(
                "separate context-menu target manifest is present but source-closure does not declare it"
                    .to_string(),
            ),
            _ => {}
        }
        if let Some(manifest) = &loaded.context_menu_target_manifest
            && let Err(error) = context_menu_manifest::validate_manifest_contract(
                manifest,
                &loaded.source_closure.root,
            )
        {
            errors.push(format!("context-menu-target-manifest: {error}"));
        }
        let leaf_ids = leaf_validation::validate(
            &loaded.action_origins,
            &loaded.branch_catalog,
            &loaded.leaf_manifest,
            mode,
            &mut errors,
        );
        execution_validation::validate(
            &loaded.execution_record,
            &loaded.storybook,
            &loaded.leaf_manifest,
            &leaf_ids,
            mode,
            &mut errors,
        );
        validate_storybook_media(
            &artifacts.storybook_artifacts,
            &loaded.storybook,
            &mut errors,
        );
        if mode == ArtifactValidationMode::KleRelease {
            kle_release_gate::validate(artifacts, &loaded, &mut errors);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("\n"))
        }
    }
}

fn validate_storybook_media(
    storybook_artifact_path: &Path,
    storybook: &StorybookArtifacts,
    errors: &mut Vec<String>,
) {
    let Some(artifact_root) = storybook_artifact_path.parent() else {
        errors.push("storybook-artifacts has no artifact root directory".to_string());
        return;
    };
    for artifact in &storybook.artifacts {
        let relative_path = Path::new(&artifact.media_path);
        if artifact.media_path.trim().is_empty()
            || relative_path.is_absolute()
            || relative_path.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            errors.push(format!(
                "storybook-artifacts story {} has invalid media path {}",
                artifact.story_id, artifact.media_path
            ));
            continue;
        }
        let media_path = artifact_root.join(relative_path);
        let metadata = match std::fs::symlink_metadata(&media_path) {
            Ok(metadata) => metadata,
            Err(error) => {
                errors.push(format!(
                    "storybook-artifacts story {} media is missing {}: {error}",
                    artifact.story_id,
                    media_path.display()
                ));
                continue;
            }
        };
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            errors.push(format!(
                "storybook-artifacts story {} media must be a regular file: {}",
                artifact.story_id,
                media_path.display()
            ));
            continue;
        }
        match std::fs::read(&media_path) {
            Ok(bytes) if !is_numbered_stage_png(relative_path, &bytes) => errors.push(format!(
                "storybook-artifacts story {} media must be a numbered PNG stage: {}",
                artifact.story_id,
                media_path.display()
            )),
            Ok(bytes) if sha256_hex(&bytes) == artifact.media_sha256 => {}
            Ok(_) => errors.push(format!(
                "storybook-artifacts story {} media hash does not match {}",
                artifact.story_id,
                media_path.display()
            )),
            Err(error) => errors.push(format!(
                "storybook-artifacts story {} media cannot be read {}: {error}",
                artifact.story_id,
                media_path.display()
            )),
        }
    }
}
