use crate::source_evidence::KatanaSourceEvidence;

use super::context_menu_contract_types::{ContextEvidenceKind, ContextMenuContractLeaf};

pub(super) const CONTEXT_MENU_HOST_E2E_SOURCE: &str =
    "tools/katana-host-e2e/tests/context_menu_input.rs";
pub(super) const CONTEXT_MENU_HOST_E2E_RUNTIME_SOURCE: &str =
    "tools/katana-host-e2e/src/context_menu_input.rs";

#[derive(Clone, Copy)]
pub(super) struct LeafSpec {
    pub(super) id: &'static str,
    pub(super) parent_path: &'static str,
    pub(super) root_slot: Option<usize>,
    pub(super) path: &'static str,
    pub(super) line: usize,
    pub(super) marker: &'static str,
}

impl LeafSpec {
    pub(super) fn leaf(self) -> ContextMenuContractLeaf {
        ContextMenuContractLeaf {
            id: self.id,
            parent_path: self.parent_path,
            root_slot: self.root_slot,
            source: KatanaSourceEvidence {
                path: self.path,
                line: self.line,
                marker: self.marker,
            },
            evidence_kind: ContextEvidenceKind::ActualHostE2e,
            host_e2e_test_source_path: CONTEXT_MENU_HOST_E2E_SOURCE,
            host_e2e_runtime_source_path: CONTEXT_MENU_HOST_E2E_RUNTIME_SOURCE,
        }
    }
}
