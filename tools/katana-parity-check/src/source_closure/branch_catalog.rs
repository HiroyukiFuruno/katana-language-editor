#[path = "branch_catalog/record.rs"]
mod record;
#[path = "branch_catalog/source.rs"]
mod source;

use super::model::{BranchCatalogArtifact, ManifestRoot, ProfileRecord};
use super::scan_state::{ControlFlowKind, ScanState};

pub(super) fn is_branch_kind(kind: &str) -> bool {
    ControlFlowKind::parse(kind).is_some()
}

pub(super) fn materialize_branch_catalog(
    root: ManifestRoot,
    state: &ScanState,
    profiles: &[ProfileRecord],
    katana_root: &std::path::Path,
) -> Result<BranchCatalogArtifact, String> {
    let mut branches = state
        .edges
        .iter()
        .filter(|edge| is_branch_kind(&edge.kind))
        .map(|edge| record::from_edge(edge, profiles, katana_root))
        .collect::<Result<Vec<_>, _>>()?;

    branches.sort_by(|left, right| left.branch_id.cmp(&right.branch_id));
    let unclassified_branch_ids = branches
        .iter()
        .map(|branch| branch.branch_id.clone())
        .collect();
    Ok(BranchCatalogArtifact {
        root,
        branches,
        unclassified_branch_ids,
    })
}
