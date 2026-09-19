use std::path::Path;

#[cfg(test)]
use super::context_menu_contract_types::ContextMenuContractLeaf;
use super::{
    context_menu_contract_declaration::ContextMenuDeclarationValidator,
    context_menu_contract_entries::ContextMenuContractEntries,
    context_menu_contract_host_e2e::ContextMenuHostE2eValidator,
};

pub(crate) struct ContextMenuContractAudit;

impl ContextMenuContractAudit {
    pub(crate) fn validate(repo_root: &Result<std::path::PathBuf, String>) -> Result<(), String> {
        let leaves = ContextMenuContractEntries::leaves();
        ContextMenuDeclarationValidator::validate(&leaves)?;
        let katana_root = repo_root
            .as_ref()
            .map_err(|error| format!("failed to resolve KatanA reference: {error}"))?;
        for leaf in &leaves {
            ContextMenuDeclarationValidator::validate_source(leaf, katana_root)?;
        }
        ContextMenuHostE2eValidator::validate(
            &leaves,
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
        )
    }

    #[cfg(test)]
    pub(crate) fn validate_declarations(leaves: &[ContextMenuContractLeaf]) -> Result<(), String> {
        ContextMenuDeclarationValidator::validate(leaves)
    }

    #[cfg(test)]
    pub(crate) fn validate_source(
        leaf: &ContextMenuContractLeaf,
        katana_root: &Path,
    ) -> Result<(), String> {
        ContextMenuDeclarationValidator::validate_source(leaf, katana_root)
    }

    #[cfg(test)]
    pub(crate) fn validate_actual_input_contract_at(
        leaves: &[ContextMenuContractLeaf],
        root: &Path,
    ) -> Result<(), String> {
        ContextMenuHostE2eValidator::validate(leaves, root)
    }
}
