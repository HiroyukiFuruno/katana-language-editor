use serde::{Deserialize, Serialize};

pub(crate) const SOURCE_CLOSURE_ARTIFACT: &str = "source-closure.json";
pub(crate) const ACTION_ORIGINS_ARTIFACT: &str = "action-origins.json";
pub(crate) const BRANCH_CATALOG_ARTIFACT: &str = "branch-catalog.json";
pub(crate) const LEAF_MANIFEST_ARTIFACT: &str = "leaf-manifest.json";
pub(crate) const EXECUTION_RECORD_ARTIFACT: &str = "execution-record.json";
pub(crate) const STORYBOOK_ARTIFACT_ARTIFACT: &str = "storybook-artifacts.json";
pub(crate) const SOURCE_DERIVED_NATIVE_TARGET_ARTIFACT: &str = "source-derived-native-target.json";
pub(crate) const CONTEXT_MENU_TARGET_MANIFEST_ARTIFACT: &str = "context-menu-target-manifest.json";
pub(crate) const KLE_RELEASE_EVIDENCE_ARTIFACT: &str = "kle-release-evidence.json";

pub(crate) const CANONICAL_PROFILE_IDS: [&str; 3] =
    ["macos-latest", "windows-latest", "ubuntu-latest"];

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ManifestRoot {
    pub(crate) schema_version: String,
    pub(crate) katana_revision: String,
    pub(crate) katana_tree_fingerprint: String,
    pub(crate) katana_external_ui_fingerprint: String,
    pub(crate) user_mandated_extensions_fingerprint: String,
    pub(crate) source_universe_fingerprint: String,
    pub(crate) requirement_source_aliases_fingerprint: String,
    pub(crate) kle_tree_fingerprint: String,
    pub(crate) kuc_tree_fingerprint: String,
    pub(crate) release_profile_matrix_fingerprint: String,
    pub(crate) generator_fingerprint: String,
    pub(crate) generated_at_utc: String,

    #[serde(default)]
    pub(crate) static_leaf_count: Option<u64>,
    #[serde(default)]
    pub(crate) expected_leaf_ids: Option<Vec<String>>,
}
