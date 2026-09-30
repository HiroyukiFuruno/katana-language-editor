use std::path::PathBuf;

use super::model::{
    ACTION_ORIGINS_ARTIFACT, BRANCH_CATALOG_ARTIFACT, CONTEXT_MENU_TARGET_MANIFEST_ARTIFACT,
    EXECUTION_RECORD_ARTIFACT, KLE_RELEASE_EVIDENCE_ARTIFACT, LEAF_MANIFEST_ARTIFACT,
    SOURCE_CLOSURE_ARTIFACT, STORYBOOK_ARTIFACT_ARTIFACT,
};

#[derive(Debug)]
pub struct ArtifactPaths {
    pub(crate) source_closure: PathBuf,
    pub(crate) action_origins: PathBuf,
    pub(crate) branch_catalog: PathBuf,
    pub(crate) leaf_manifest: PathBuf,
    pub(crate) execution_record: PathBuf,
    pub(crate) storybook_artifacts: PathBuf,
    pub(crate) context_menu_target_manifest: PathBuf,
    pub(crate) kle_release_evidence: PathBuf,
}

impl ArtifactPaths {
    pub(crate) fn from_dir(dir: impl Into<PathBuf>) -> Self {
        let dir = dir.into();
        Self {
            source_closure: dir.join(SOURCE_CLOSURE_ARTIFACT),
            action_origins: dir.join(ACTION_ORIGINS_ARTIFACT),
            branch_catalog: dir.join(BRANCH_CATALOG_ARTIFACT),
            leaf_manifest: dir.join(LEAF_MANIFEST_ARTIFACT),
            execution_record: dir.join(EXECUTION_RECORD_ARTIFACT),
            storybook_artifacts: dir.join(STORYBOOK_ARTIFACT_ARTIFACT),
            context_menu_target_manifest: dir.join(CONTEXT_MENU_TARGET_MANIFEST_ARTIFACT),
            kle_release_evidence: dir.join(KLE_RELEASE_EVIDENCE_ARTIFACT),
        }
    }
}
