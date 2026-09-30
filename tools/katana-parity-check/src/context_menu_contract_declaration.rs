use std::{collections::BTreeSet, path::Path};

use crate::source_inventory_repo::SourceInventoryRepo;

use super::{
    context_menu_contract_inventory::ContextMenuInventoryValidator,
    context_menu_contract_types::{ContextEvidenceKind, ContextMenuContractLeaf},
};

pub(super) struct ContextMenuDeclarationValidator;

impl ContextMenuDeclarationValidator {
    pub(super) fn validate(leaves: &[ContextMenuContractLeaf]) -> Result<(), String> {
        let mut ids = BTreeSet::new();
        let mut root_slots = Vec::new();
        for leaf in leaves {
            Self::validate_leaf(leaf, &mut ids, &mut root_slots)?;
        }
        ContextMenuInventoryValidator::validate(&ids, &root_slots)
    }

    pub(super) fn validate_source(
        leaf: &ContextMenuContractLeaf,
        katana_root: &Path,
    ) -> Result<(), String> {
        let source =
            SourceInventoryRepo::read_file(&katana_root.join(leaf.source.path)).map_err(|_| {
                format!(
                    "missing context-menu structural source: {}",
                    leaf.source.path
                )
            })?;
        let line = source.lines().nth(leaf.source.line - 1).ok_or_else(|| {
            format!(
                "missing context-menu structural source line: {}:{}",
                leaf.source.path, leaf.source.line
            )
        })?;
        line.contains(leaf.source.marker)
            .then_some(())
            .ok_or_else(|| {
                format!(
                    "context-menu structural source marker mismatch: {}:{} expected {:?}",
                    leaf.source.path, leaf.source.line, leaf.source.marker
                )
            })
    }

    fn validate_leaf(
        leaf: &ContextMenuContractLeaf,
        ids: &mut BTreeSet<&'static str>,
        root_slots: &mut Vec<(Option<usize>, &'static str)>,
    ) -> Result<(), String> {
        if !ids.insert(leaf.id) {
            return Err(format!(
                "duplicate context-menu structural leaf: {}",
                leaf.id
            ));
        }
        if leaf.parent_path.is_empty()
            || leaf.source.path.is_empty()
            || leaf.source.line == 0
            || leaf.source.marker.is_empty()
        {
            return Err(format!(
                "incomplete context-menu structural leaf: {}",
                leaf.id
            ));
        }
        if let Some(expected_path) = ContextMenuInventoryValidator::expected_parent_path(leaf.id)
            && leaf.parent_path != expected_path
        {
            return Err(format!(
                "context-menu structural parent path mismatch for {}: expected {}",
                leaf.id, expected_path
            ));
        }
        if leaf.evidence_kind != ContextEvidenceKind::ActualHostE2e {
            return Err(format!(
                "context-menu structural leaf has non-actual evidence: {}",
                leaf.id
            ));
        }
        Self::validate_host_e2e_paths(leaf)?;
        if leaf.id.starts_with("context.structure.root.") {
            root_slots.push((leaf.root_slot, leaf.id));
        }
        Ok(())
    }

    fn validate_host_e2e_paths(leaf: &ContextMenuContractLeaf) -> Result<(), String> {
        use super::context_menu_contract_entry_types::{
            CONTEXT_MENU_HOST_E2E_RUNTIME_SOURCE, CONTEXT_MENU_HOST_E2E_SOURCE,
        };
        (leaf.host_e2e_test_source_path == CONTEXT_MENU_HOST_E2E_SOURCE
            && leaf.host_e2e_runtime_source_path == CONTEXT_MENU_HOST_E2E_RUNTIME_SOURCE)
            .then_some(())
            .ok_or_else(|| {
                format!(
                    "context-menu structural leaf must require the ContextMenu host E2E actual-input evidence: {}",
                    leaf.id
                )
            })
    }
}
