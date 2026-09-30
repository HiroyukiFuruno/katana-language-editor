#[path = "leaf_inventory_validation_audit.rs"]
mod audit;
#[path = "leaf_inventory_validation_inventory.rs"]
mod inventory;
#[path = "leaf_inventory_validation_leaf.rs"]
mod leaf;
pub(super) fn validate_inventory(
    leaves: &[super::leaf_inventory_types::LeafCapability],
) -> Result<(), String> {
    inventory::validate_inventory(leaves)
}

pub(super) fn validate_leaf(
    leaf: &super::leaf_inventory_types::LeafCapability,
    katana_root: &std::path::Path,
    kle_root: &std::path::Path,
) -> Result<(), String> {
    leaf::validate_leaf(leaf, katana_root, kle_root)
}

pub(super) fn kle_workspace_root() -> Result<std::path::PathBuf, String> {
    leaf::kle_workspace_root()
}

#[cfg(test)]
pub(super) fn validate_inventory_shape(
    leaves: &[super::leaf_inventory_types::LeafCapability],
) -> Result<(), String> {
    inventory::validate_inventory_shape(leaves)
}

#[cfg(test)]
mod tests {
    use super::validate_inventory_shape;

    #[test]
    fn validates_inventory_shape_through_public_validation_wrapper() -> Result<(), String> {
        match validate_inventory_shape(&[]) {
            Err(error) if error.contains("exact editor leaf capabilities") => Ok(()),
            Ok(()) => Err("empty inventory was accepted".to_string()),
            Err(error) => Err(format!("unexpected rejection: {error}")),
        }
    }
}
