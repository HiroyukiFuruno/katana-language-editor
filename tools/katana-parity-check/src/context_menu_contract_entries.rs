use std::collections::BTreeSet;

use super::{
    context_menu_contract_entries_code::CODE_SPECS,
    context_menu_contract_entries_root::ROOT_AND_EDIT_SPECS,
    context_menu_contract_entries_routes::ROUTE_AND_INGEST_SPECS,
    context_menu_contract_types::ContextMenuContractLeaf,
};

pub(crate) struct ContextMenuContractEntries;

impl ContextMenuContractEntries {
    pub(crate) fn leaves() -> Vec<ContextMenuContractLeaf> {
        Self::all_specs()
            .iter()
            .copied()
            .map(|spec| spec.leaf())
            .collect()
    }

    pub(crate) fn expected_leaf_ids() -> BTreeSet<&'static str> {
        Self::all_specs().iter().map(|spec| spec.id).collect()
    }

    fn all_specs() -> Vec<super::context_menu_contract_entry_types::LeafSpec> {
        let mut specs = ROOT_AND_EDIT_SPECS.to_vec();
        specs.extend_from_slice(CODE_SPECS);
        specs.extend_from_slice(ROUTE_AND_INGEST_SPECS);
        specs
    }
}
