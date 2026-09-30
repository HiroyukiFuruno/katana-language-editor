use super::super::leaf_inventory_entries::LEAVES;
use super::{kle_workspace_root, validate_inventory, validate_leaf};
use crate::capability_manifest::CapabilityManifestAudit;
use std::path::PathBuf;
pub(crate) struct LeafCapabilityAudit;

impl LeafCapabilityAudit {
    pub(crate) fn validate(repo_root: &Result<PathBuf, String>) -> Result<(), String> {
        validate_inventory(LEAVES)?;
        let katana_root = repo_root
            .as_ref()
            .map_err(|error| format!("failed to resolve KatanA reference: {error}"))?;
        let kle_root = kle_workspace_root()?;
        let mut failures = Vec::new();
        for leaf in LEAVES {
            if let Err(error) = validate_leaf(leaf, katana_root, &kle_root) {
                failures.push(format!("{}: {error}", leaf.id));
            }
        }
        failures
            .is_empty()
            .then_some(())
            .ok_or_else(|| failures.join("; "))
    }

    pub(crate) fn validate_host_e2e_execution() -> Result<(), String> {
        let evidence = LEAVES.iter().map(|leaf| leaf.host_e2e).collect::<Vec<_>>();
        CapabilityManifestAudit::validate_host_e2e_execution(&evidence)
    }
}
