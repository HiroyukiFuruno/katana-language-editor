use serde::{Deserialize, Serialize};

use super::operational_input::InactiveCfgEdge;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ProfileRecord {
    pub(crate) id: String,
    pub(crate) runner_label: String,
    pub(crate) katana_revision: String,
    pub(crate) fingerprint: String,
    pub(crate) rustc_host_triple: String,
    pub(crate) rustc_cfg_sha256: String,
    pub(crate) cargo_resolution_sha256: String,
    pub(crate) lockfile_sha256: String,
    pub(crate) active_edge_ids: Vec<String>,
    pub(crate) inactive_cfg_edges: Vec<InactiveCfgEdge>,
}
