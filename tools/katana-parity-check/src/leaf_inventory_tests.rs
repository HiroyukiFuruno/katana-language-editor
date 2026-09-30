use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::capability_manifest::{
    HostEffectKind, KatanaSourceEvidence, KleActualFrameHarness, KleHostE2eEvidence,
    KleHostE2eTestLocator, KleSourceLocator,
};

use super::{
    leaf_inventory_entries::{LEAVES, REQUIRED_LEAF_COUNT, TOOLBAR},
    leaf_inventory_types::{
        LeafCapability, LeafEvidenceKind, LeafKleEvidence, OWNERS, REJECTED_KLE_EVIDENCE_KINDS,
    },
    leaf_inventory_validation::{validate_inventory, validate_inventory_shape, validate_leaf},
};

#[path = "leaf_inventory_tests_rejections.rs"]
mod leaf_inventory_tests_rejections;

static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);

const FIXTURE_HARNESS: KleActualFrameHarness = KleActualFrameHarness {
    public_show_callsite: KleSourceLocator {
        source_path: "tests/harness.rs",
        line: 4,
        marker: "context.run_ui(input(events), |ui| result = Some(editor.show(ui)));",
    },
    raw_input_root: KleSourceLocator {
        source_path: "tests/harness.rs",
        line: 8,
        marker: "fn input(events: Vec<Event>) -> RawInput {",
    },
    raw_input_construction: KleSourceLocator {
        source_path: "tests/harness.rs",
        line: 9,
        marker: "RawInput {",
    },
    scenario_implementation: KleSourceLocator {
        source_path: "tests/harness.rs",
        line: 2,
        marker: "impl Scenario {",
    },
    symbol: "Scenario",
    call_path: "Scenario::run",
};
pub(super) const FIXTURE_LEAF: LeafCapability = LeafCapability {
    parent_group: TOOLBAR,
    id: "toolbar.bold",
    katana_source_symbol: "MarkdownAuthoringOp::Bold",
    source: KatanaSourceEvidence {
        path: "src/editor.rs",
        line: 1,
        marker: "marker",
    },
    owners: OWNERS,
    kle: LeafKleEvidence {
        source_path: "tests/leaf.rs",
        selector: "public_show_leaf",
        kind: LeafEvidenceKind::ActualKleInput,
        harness: FIXTURE_HARNESS,
    },
    visible_menu_path: "public_show_toolbar_routes_all_actions_and_lifecycle",
    state_condition: "always",
    typed_kle_request_or_state: "MarkdownAuthoringOp::Bold",
    expected_actual_katana_effect: HostEffectKind::ContextAuthoring.description(),
    host_e2e: KleHostE2eEvidence {
        test: KleHostE2eTestLocator {
            target: "actual_host",
            source_path: "tools/katana-host-e2e/tests/actual_host.rs",
            selector: "actual_kle_fixture_context_authoring",
        },
        effect: HostEffectKind::ContextAuthoring,
    },
};

pub(super) struct Fixture {
    pub(super) katana_root: PathBuf,
    pub(super) kle_root: PathBuf,
}

impl Fixture {
    pub(super) fn new() -> Self {
        let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "katana-parity-leaf-{}-{sequence}",
            std::process::id()
        ));
        let katana_root = root.join("katana");
        let kle_root = root.join("kle");
        fs::create_dir_all(katana_root.join("src")).unwrap();
        fs::create_dir_all(kle_root.join("tests")).unwrap();
        let fixture = Self {
            katana_root,
            kle_root,
        };
        fixture.write_valid();
        fixture
    }

    fn write_valid(&self) {
        fs::write(self.katana_root.join("src/editor.rs"), "marker\n").unwrap();
        fs::write(
            self.kle_root.join("tests/harness.rs"),
            fixture_harness_source(),
        )
        .unwrap();
        fs::write(
            self.kle_root.join("tests/leaf.rs"),
            fixture_leaf_test_source(true, true, true),
        )
        .unwrap();
        fs::create_dir_all(self.kle_root.join("tools/katana-host-e2e/tests")).unwrap();
        fs::write(
            self.kle_root.join("tools/katana-host-e2e/Cargo.toml"),
            "[package]\nname = \"katana-host-e2e\"\n",
        )
        .unwrap();
        fs::write(
            self.kle_root
                .join("tools/katana-host-e2e/tests/actual_host.rs"),
            fixture_host_e2e_source(),
        )
        .unwrap();
        fs::create_dir_all(self.kle_root.join("tools/katana-host-e2e/src")).unwrap();
        fs::write(
            self.kle_root.join("tools/katana-host-e2e/src/host.rs"),
            fixture_host_runtime_source(),
        )
        .unwrap();
    }
}

fn fixture_host_e2e_source() -> &'static str {
    "#[test]\nfn actual_kle_fixture_context_authoring() { host_effect(); }\nfn host_effect() { ContextMenuPhysicalInput::case_specs(); let authoring_specs = vec![()]; if authoring_specs.len() != 30 {} assert_exact_authoring_document_effect(); KatanaHost::run_editor_request(); KatanaHost::run_ui_frame(); let _ = core::mem::size_of::<katana_ui::shell::KatanaApp>(); if document.buffer != expected.buffer || !document.is_dirty {} }\nfn assert_exact_authoring_document_effect() {}\n"
}

fn fixture_host_runtime_source() -> &'static str {
    "impl KatanaHost { fn run_editor_request_with_context() { app.trigger_action(action); Self::run_ui_frame_with_context(); } }\n"
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let Some(root) = self.katana_root.parent() else {
            return;
        };
        let _ = fs::remove_dir_all(root);
    }
}

fn fixture_harness_source() -> &'static str {
    "use crate::EguiLanguageEditor;\nimpl Scenario {\nfn run(editor: &mut EguiLanguageEditor, context: &Context, events: Vec<Event>) {\ncontext.run_ui(input(events), |ui| result = Some(editor.show(ui)));\n}\nfn other() {}\n}\nfn input(events: Vec<Event>) -> RawInput {\nRawInput {\n}\n}\n"
}

pub(super) fn fixture_leaf_test_source(
    include_leaf_assertion: bool,
    include_artifact_assertion: bool,
    include_accesskit_assertion: bool,
) -> String {
    let leaf_assertion = if include_leaf_assertion {
        "assert_leaf_case(\"toolbar.bold\");\n"
    } else {
        ""
    };
    let artifact_assertion = if include_artifact_assertion {
        "assert_toolbar_artifact_leaf(\"toolbar.bold\");\n"
    } else {
        ""
    };
    let accesskit_assertion = if include_accesskit_assertion {
        "assert_toolbar_accesskit_leaf(\"toolbar.bold\");\n"
    } else {
        ""
    };
    format!(
        "#[test]\nfn public_show_leaf() {{\nScenario::run();\n{leaf_assertion}{artifact_assertion}{accesskit_assertion}}}\n"
    )
}

#[test]
fn inventory_has_the_exact_required_leaf_set() {
    validate_inventory_shape(LEAVES).unwrap();
}

#[test]
fn rejects_duplicate_leaf() {
    let mut leaves = LEAVES.to_vec();
    leaves[1].id = leaves[0].id;
    assert!(
        validate_inventory(&leaves)
            .unwrap_err()
            .contains("duplicate")
    );
}

#[test]
fn rejects_non_actual_leaf_evidence() {
    let mut leaves = LEAVES.to_vec();
    leaves[0].kle.kind = LeafEvidenceKind::StorybookOnly;
    assert!(
        validate_inventory(&leaves)
            .unwrap_err()
            .contains("non-actual KLE evidence")
    );
}

#[test]
fn rejects_every_non_actual_leaf_evidence_kind() {
    for kind in REJECTED_KLE_EVIDENCE_KINDS {
        let mut leaves = LEAVES.to_vec();
        leaves[0].kle.kind = *kind;
        assert!(
            validate_inventory(&leaves)
                .unwrap_err()
                .contains("non-actual KLE evidence")
        );
    }
}

#[test]
fn rejects_missing_leaf_count() {
    assert!(
        validate_inventory(&LEAVES[..REQUIRED_LEAF_COUNT - 1])
            .unwrap_err()
            .contains("expected")
    );
}

#[test]
fn rejects_unexpected_leaf_id_even_when_count_is_preserved() {
    let mut leaves = LEAVES.to_vec();
    leaves[0].id = "toolbar.unexpected";
    assert!(
        validate_inventory(&leaves)
            .unwrap_err()
            .contains("IDs do not match")
    );
}

#[test]
fn accepts_a_leaf_with_actual_raw_input_show_and_exact_case_assertions() {
    let fixture = Fixture::new();
    validate_leaf(&FIXTURE_LEAF, &fixture.katana_root, &fixture.kle_root).unwrap();
}
