use serde::{Deserialize, Serialize};

use super::root_model::ManifestRoot;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ExecutionKind {
    #[default]
    KatanaHost,
    KucConsumerArtifact,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExecutionRecord {
    #[serde(default)]
    pub(crate) execution_kind: ExecutionKind,
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
