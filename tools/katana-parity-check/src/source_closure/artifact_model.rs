use serde::{Deserialize, Serialize};

use super::canonical_leaf_binding::CanonicalLeafBindingEvidence;
use super::context_menu_manifest::ContextMenuTargetManifest;
use super::edge_model::{ClosureEdge, ExternalUiDependency, SourceClosureFile};
use super::root_model::ManifestRoot;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourceDerivedNativeTargetRecord {
    pub(crate) schema_version: String,
    pub(crate) generated_by: String,
    pub(crate) katana_revision: String,
    pub(crate) profile_fingerprint: String,
    pub(crate) source_span_digest: String,
    pub(crate) role_digest: String,
    pub(crate) name_digests: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceClosureArtifact {
    #[serde(flatten)]
    pub(crate) root: ManifestRoot,
    pub(crate) profiles: Vec<super::profile_model::ProfileRecord>,
    pub(crate) files: Vec<SourceClosureFile>,
    pub(crate) external_ui_semantic_dependencies: Vec<ExternalUiDependency>,
    pub(crate) unresolved_edges: Vec<ClosureEdge>,
    #[serde(default)]
    pub(crate) source_derived_native_target: Option<SourceDerivedNativeTargetRecord>,
    #[serde(default)]
    pub(crate) context_menu_target_manifest: Option<ContextMenuTargetManifest>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionOrigin {
    pub(crate) action_id: String,
    pub(crate) definition_span: String,
    pub(crate) construction_sites: Vec<String>,
    pub(crate) origin_classification: String,
    pub(crate) active_profile_ids: Vec<String>,
    pub(crate) origin_input: Option<ActionOriginInput>,
    pub(crate) forward_route: Vec<String>,
    pub(crate) kle_leafs: Vec<String>,
    pub(crate) fingerprint: String,
    pub(crate) predecessor_actions: Option<Vec<String>>,
    pub(crate) state_span: Option<String>,
    pub(crate) no_renderer_or_shortcut_rationale: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ActionOriginInput {
    pub(crate) kind: String,
    pub(crate) source_span: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionOriginsArtifact {
    #[serde(flatten)]
    pub(crate) root: ManifestRoot,
    pub(crate) actions: Vec<ActionOrigin>,
    pub(crate) unresolved_action_origins: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BranchRecord {
    pub(crate) branch_id: String,
    pub(crate) file: String,
    pub(crate) symbol: String,
    pub(crate) span: TextSpan,
    pub(crate) kind: String,
    pub(crate) condition: String,
    pub(crate) active_profile_ids: Vec<String>,
    pub(crate) inactive_profile_predicates: Vec<InactiveProfilePredicate>,
    pub(crate) outcomes: Vec<String>,
    pub(crate) incoming_edges: Vec<String>,
    pub(crate) source_excerpt_sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct TextSpan {
    pub(crate) start_line: usize,
    pub(crate) end_line: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct InactiveProfilePredicate {
    pub(crate) profile_id: String,
    pub(crate) predicate: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BranchCatalogArtifact {
    #[serde(flatten)]
    pub(crate) root: ManifestRoot,
    pub(crate) branches: Vec<BranchRecord>,
    pub(crate) unclassified_branch_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequirementOrigin {
    pub(crate) kind: String,
    pub(crate) span: String,
}

pub(crate) const KATANA_ADOPTION_ISSUE_URL: &str =
    "https://github.com/HiroyukiFuruno/KatanA/issues/336";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum KatanaHostE2eState {
    DownstreamRequired,
    Passing,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct KatanaHostE2eRequirement {
    pub(crate) state: KatanaHostE2eState,
    pub(crate) adoption_issue_url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeafStatusRecord {
    pub(crate) leaf_id: String,
    pub(crate) source_candidate_fingerprint: String,
    pub(crate) requirement_origin: RequirementOrigin,
    pub(crate) source_branches: Vec<String>,
    pub(crate) action_origin_ids: Vec<String>,
    pub(crate) required_profile_ids: Vec<String>,
    pub(crate) visible_path: Vec<String>,
    pub(crate) state_preconditions: Vec<String>,
    pub(crate) kuc_component: String,
    pub(crate) kle_public_show_signature: String,
    pub(crate) kle_opaque_transit: String,
    pub(crate) declared_effect_kind: String,
    pub(crate) host_e2e: KatanaHostE2eRequirement,
    pub(crate) execution_id: Option<String>,
    pub(crate) storybook_stage_id: String,
    pub(crate) status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeafManifestArtifact {
    #[serde(flatten)]
    pub(crate) root: ManifestRoot,
    pub(crate) canonical_binding: CanonicalLeafBindingEvidence,
    pub(crate) leafs: Vec<LeafStatusRecord>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ExecutionRecord {
    pub(crate) execution_id: String,
    pub(crate) execution_profile_id: String,
    pub(crate) leaf_id: String,
    pub(crate) action_route_signature: String,
    pub(crate) status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExecutionRecordArtifact {
    #[serde(flatten)]
    pub(crate) root: ManifestRoot,
    pub(crate) executions: Vec<ExecutionRecord>,
    pub(crate) stale_or_missing_leafs: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct StorybookArtifactEntry {
    pub(crate) story_id: String,
    pub(crate) leaf_id: String,
    pub(crate) stage_id: String,
    pub(crate) frame_record_id: String,
    pub(crate) media_path: String,
    pub(crate) media_sha256: String,
    pub(crate) profile_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StorybookArtifacts {
    #[serde(flatten)]
    pub(crate) root: ManifestRoot,
    pub(crate) artifacts: Vec<StorybookArtifactEntry>,
}
