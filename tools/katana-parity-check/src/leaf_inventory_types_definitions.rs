use crate::capability_manifest::{
    CapabilityOwner, KatanaSourceEvidence, KleActualFrameHarness, KleHostE2eEvidence,
    KleSourceLocator,
};

pub(crate) const OWNERS: &[CapabilityOwner] = &[
    CapabilityOwner::KucRuntime,
    CapabilityOwner::KleBinding,
    CapabilityOwner::KatanaHost,
];

pub(crate) const LEAF_DIRECT_TEXT_SURFACE_HARNESS: KleActualFrameHarness = KleActualFrameHarness {
    public_show_callsite: KleSourceLocator {
        source_path: "crates/katana-language-editor-egui/src/direct_text_surface_actual_input_context_menu.rs",
        line: 103,
        marker: "context.run_ui(raw_input(events), |ui| result = Some(editor.show(ui)))",
    },
    raw_input_root: KleSourceLocator {
        source_path: "crates/katana-language-editor-egui/src/direct_text_surface_actual_input_context_menu.rs",
        line: 731,
        marker: "fn raw_input(events: Vec<Event>) -> RawInput {",
    },
    raw_input_construction: KleSourceLocator {
        source_path: "crates/katana-language-editor-egui/src/direct_text_surface_actual_input_context_menu.rs",
        line: 732,
        marker: "RawInput {",
    },
    scenario_implementation: KleSourceLocator {
        source_path: "crates/katana-language-editor-egui/src/direct_text_surface_actual_input_context_menu.rs",
        line: 49,
        marker: "impl ContextMenuRawInputScenario {",
    },
    symbol: "ContextMenuRawInputScenario",
    call_path: "ContextMenuRawInputScenario::run",
};

pub(crate) const LEAF_COMMAND_CHROME_HARNESS: KleActualFrameHarness = KleActualFrameHarness {
    public_show_callsite: KleSourceLocator {
        source_path: "crates/katana-language-editor-egui/src/kuc_command_chrome_composition_f2_harness.rs",
        line: 17,
        marker: "context.run_ui(Self::raw_input(events), |ui| result = Some(editor.show(ui)))",
    },
    raw_input_root: KleSourceLocator {
        source_path: "crates/katana-language-editor-egui/src/kuc_command_chrome_composition_f2_harness.rs",
        line: 73,
        marker: "fn raw_input(events: Vec<Event>) -> RawInput {",
    },
    raw_input_construction: KleSourceLocator {
        source_path: "crates/katana-language-editor-egui/src/kuc_command_chrome_composition_f2_harness.rs",
        line: 74,
        marker: "RawInput {",
    },
    scenario_implementation: KleSourceLocator {
        source_path: "crates/katana-language-editor-egui/src/kuc_command_chrome_composition_f2_scenario.rs",
        line: 12,
        marker: "impl F2CommandChromeScenario {",
    },
    symbol: "F2CommandChromeScenario",
    call_path: "F2CommandChromeScenario::run_command_chrome",
};

#[derive(Clone, Copy)]
pub(crate) enum LeafEvidenceKind {
    ActualKleInput,
    SourceOnly,
    StorybookOnly,
    SimulatorOnly,
}

pub(crate) const REJECTED_KLE_EVIDENCE_KINDS: &[LeafEvidenceKind] = &[
    LeafEvidenceKind::SourceOnly,
    LeafEvidenceKind::StorybookOnly,
    LeafEvidenceKind::SimulatorOnly,
];
pub(crate) const MISSING_LEAF_SCHEMA_FIELD: &str = "__MISSING_LEAF_SCHEMA_FIELD__";

#[derive(Clone, Copy)]
pub(crate) struct LeafKleEvidence {
    pub(crate) source_path: &'static str,
    pub(crate) selector: &'static str,
    pub(crate) kind: LeafEvidenceKind,
    pub(crate) harness: KleActualFrameHarness,
}

#[derive(Clone, Copy)]
pub(crate) struct LeafCapability {
    pub(crate) parent_group: &'static str,
    pub(crate) id: &'static str,
    pub(crate) katana_source_symbol: &'static str,
    pub(crate) source: KatanaSourceEvidence,
    pub(crate) owners: &'static [CapabilityOwner],
    pub(crate) kle: LeafKleEvidence,
    pub(crate) visible_menu_path: &'static str,
    pub(crate) state_condition: &'static str,
    pub(crate) typed_kle_request_or_state: &'static str,
    pub(crate) expected_actual_katana_effect: &'static str,
    pub(crate) host_e2e: KleHostE2eEvidence,
}

pub(super) const fn manifest_visible_menu_path(selector: &'static str) -> &'static str {
    if selector.is_empty() {
        MISSING_LEAF_SCHEMA_FIELD
    } else {
        selector
    }
}

pub(super) const fn manifest_state_condition() -> &'static str {
    "always"
}

pub(crate) struct LeafDeclaration {
    pub(crate) parent_group: &'static str,
    pub(crate) id: &'static str,
    pub(crate) path: &'static str,
    pub(crate) line: usize,
    pub(crate) marker: &'static str,
    pub(crate) selector: &'static str,
}
