#[path = "branch_catalog/record.rs"]
mod record;
#[path = "branch_catalog/source.rs"]
mod source;

use super::model::{BranchCatalogArtifact, ManifestRoot, ProfileRecord};
use super::scan_state::ScanState;

pub(super) fn is_branch_kind(kind: &str) -> bool {
    matches!(
        kind,
        "cfg"
            | "if"
            | "if_then"
            | "if_else"
            | "match"
            | "match_guard"
            | "match_body"
            | "let_else"
            | "let_else_body"
            | "logical_and"
            | "logical_or"
            | "for"
            | "for_body"
            | "while"
            | "while_body"
            | "loop"
            | "loop_body"
            | "return"
            | "break"
            | "continue"
            | "try"
    )
}

pub(super) fn materialize_branch_catalog(
    root: ManifestRoot,
    state: &ScanState,
    profiles: &[ProfileRecord],
    katana_root: &std::path::Path,
) -> Result<BranchCatalogArtifact, String> {
    let profile_ids = profiles
        .iter()
        .map(|profile| profile.id.clone())
        .collect::<Vec<_>>();
    let mut branches = state
        .edges
        .iter()
        .filter(|edge| is_branch_kind(&edge.kind))
        .map(|edge| record::from_edge(edge, profiles, &profile_ids, katana_root))
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
