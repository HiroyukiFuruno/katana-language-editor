use crate::capability_manifest::{KleActualFrameHarness, KleHostE2eEvidence};

#[path = "leaf_inventory_types_builders.rs"]
mod builders;
#[path = "leaf_inventory_types_definitions.rs"]
mod definitions;
pub(crate) use definitions::*;

pub(super) const fn leaf(
    parent_group: &'static str,
    id: &'static str,
    path: &'static str,
    line: usize,
    marker: &'static str,
    selector: &'static str,
) -> LeafCapability {
    builders::leaf(parent_group, id, path, line, marker, selector)
}

pub(super) const fn leaf_with_host(
    parent_group: &'static str,
    id: &'static str,
    path: &'static str,
    line: usize,
    marker: &'static str,
    selector: &'static str,
    host_e2e: KleHostE2eEvidence,
) -> LeafCapability {
    builders::leaf_with_host(parent_group, id, path, line, marker, selector, host_e2e)
}

pub(super) const fn leaf_with_harness(
    parent_group: &'static str,
    id: &'static str,
    path: &'static str,
    line: usize,
    marker: &'static str,
    selector: &'static str,
    harness: KleActualFrameHarness,
) -> LeafCapability {
    builders::leaf_with_harness(parent_group, id, path, line, marker, selector, harness)
}

pub(super) const fn leaf_with_harness_host(
    declaration: LeafDeclaration,
    harness: KleActualFrameHarness,
    host_e2e: KleHostE2eEvidence,
) -> LeafCapability {
    builders::leaf_with_harness_host(declaration, harness, host_e2e)
}
