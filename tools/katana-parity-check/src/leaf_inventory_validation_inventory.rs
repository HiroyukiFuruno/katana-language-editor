use super::{
    super::leaf_inventory_types::LeafCapability,
    super::{
        leaf_inventory_entries::REQUIRED_LEAF_COUNT, leaf_inventory_expected_ids::EXPECTED_LEAF_IDS,
    },
    leaf::validate_leaf_declaration,
};
use std::collections::BTreeSet;
pub(super) fn validate_inventory(leaves: &[LeafCapability]) -> Result<(), String> {
    validate_inventory_shape(leaves)?;
    let mut selectors = BTreeSet::new();
    for leaf in leaves {
        validate_leaf_declaration(leaf)?;
        if !selectors.insert(leaf.kle.selector) {
            return Err(format!(
                "leaf {} shares selector {} with another leaf",
                leaf.id, leaf.kle.selector
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_inventory_shape(leaves: &[LeafCapability]) -> Result<(), String> {
    if leaves.len() != REQUIRED_LEAF_COUNT {
        return Err(format!(
            "expected {REQUIRED_LEAF_COUNT} exact editor leaf capabilities, got {}",
            leaves.len()
        ));
    }
    let mut ids = BTreeSet::new();
    for leaf in leaves {
        super::leaf::validate_leaf_shape(leaf, &mut ids)?;
    }
    let expected = EXPECTED_LEAF_IDS.iter().copied().collect::<BTreeSet<_>>();
    if ids == expected {
        Ok(())
    } else {
        Err("exact editor leaf capability IDs do not match the required inventory".to_string())
    }
}
