use std::path::{Path, PathBuf};

use super::operational_publication_copy::copy_tree;
use super::operational_publication_fingerprint::evidence_tree_fingerprint_for;
use super::root_model::SOURCE_CLOSURE_ARTIFACT;

const MATERIALIZED_DIRECTORY: &str = "materialized";
const PUBLISHED_ARTIFACT_DIRECTORY: &str = "artifacts";

pub(super) fn copy_verified_materialized_artifacts(
    staging: &Path,
    publication: &Path,
) -> Result<String, String> {
    let source = materialized_artifact_dir(staging)?;
    let source_fingerprint = evidence_tree_fingerprint_for(&source)?;
    let destination = publication.join(PUBLISHED_ARTIFACT_DIRECTORY);
    copy_tree(&source, &destination)?;
    let destination_fingerprint = evidence_tree_fingerprint_for(&destination)?;
    if source_fingerprint != destination_fingerprint {
        return Err("published materialized artifacts do not match staging".to_string());
    }
    Ok(source_fingerprint)
}

pub(super) fn verify_published_materialized_artifacts(
    staging: &Path,
    publication: &Path,
) -> Result<(), String> {
    let source = materialized_artifact_dir(staging)?;
    let source_fingerprint = evidence_tree_fingerprint_for(&source)?;
    let destination = publication.join(PUBLISHED_ARTIFACT_DIRECTORY);
    let destination_fingerprint = evidence_tree_fingerprint_for(&destination)?;
    if source_fingerprint != destination_fingerprint {
        return Err("canonical materialized artifacts do not match staging".to_string());
    }
    Ok(())
}

fn materialized_artifact_dir(staging: &Path) -> Result<PathBuf, String> {
    let directory = staging.join(MATERIALIZED_DIRECTORY);
    if !directory.join(SOURCE_CLOSURE_ARTIFACT).is_file() {
        return Err(format!(
            "materialized source-closure artifact is required before publication: {}",
            directory.join(SOURCE_CLOSURE_ARTIFACT).display()
        ));
    }
    Ok(directory)
}
