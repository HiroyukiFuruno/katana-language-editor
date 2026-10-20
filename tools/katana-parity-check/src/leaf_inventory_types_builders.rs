use super::super::leaf_inventory_host_e2e::{
    CONTEXT_AUTHORING_HOST_E2E, TOOLBAR_AUTHORING_HOST_E2E,
};
use super::{
    LEAF_DIRECT_TEXT_SURFACE_HARNESS, OWNERS,
    definitions::{
        LeafCapability, LeafDeclaration, LeafEvidenceKind, LeafKleEvidence,
        manifest_state_condition, manifest_visible_menu_path,
    },
};
use crate::capability_manifest::{KatanaSourceEvidence, KleActualFrameHarness, KleHostE2eEvidence};
pub(super) const fn leaf(
    parent_group: &'static str,
    id: &'static str,
    path: &'static str,
    line: usize,
    marker: &'static str,
    selector: &'static str,
) -> LeafCapability {
    leaf_with_harness_host(
        LeafDeclaration {
            parent_group,
            id,
            path,
            line,
            marker,
            selector,
        },
        LEAF_DIRECT_TEXT_SURFACE_HARNESS,
        CONTEXT_AUTHORING_HOST_E2E,
    )
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
    leaf_with_harness_host(
        LeafDeclaration {
            parent_group,
            id,
            path,
            line,
            marker,
            selector,
        },
        LEAF_DIRECT_TEXT_SURFACE_HARNESS,
        host_e2e,
    )
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
    leaf_with_harness_host(
        LeafDeclaration {
            parent_group,
            id,
            path,
            line,
            marker,
            selector,
        },
        harness,
        TOOLBAR_AUTHORING_HOST_E2E,
    )
}

pub(super) const fn leaf_with_harness_host(
    declaration: LeafDeclaration,
    harness: KleActualFrameHarness,
    host_e2e: KleHostE2eEvidence,
) -> LeafCapability {
    LeafCapability {
        parent_group: declaration.parent_group,
        id: declaration.id,
        katana_source_symbol: declaration.marker,
        source: KatanaSourceEvidence {
            path: declaration.path,
            line: declaration.line,
            marker: declaration.marker,
        },
        owners: OWNERS,
        kle: LeafKleEvidence {
            source_path: "crates/katana-language-editor-egui/src/direct_text_surface_integration_tests.rs",
            selector: declaration.selector,
            kind: LeafEvidenceKind::ActualKleInput,
            harness,
        },
        visible_menu_path: manifest_visible_menu_path(declaration.selector),
        state_condition: manifest_state_condition(),
        typed_kle_request_or_state: declaration.marker,
        expected_actual_katana_effect: host_e2e.effect.description(),
        host_e2e,
    }
}
