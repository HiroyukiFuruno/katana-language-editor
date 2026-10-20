use std::fs;

use crate::capability_manifest::HostEffectKind;

use super::super::{FIXTURE_LEAF, Fixture, fixture_leaf_test_source, validate_leaf};
#[test]
fn rejects_leaf_without_kle_raw_input_root() {
    let fixture = Fixture::new();
    let mut leaf = FIXTURE_LEAF;
    leaf.kle.harness.raw_input_root.line = 1;
    leaf.kle.harness.raw_input_root.marker = "missing raw input root";
    let error = validate_leaf(&leaf, &fixture.katana_root, &fixture.kle_root).unwrap_err();
    assert!(error.contains(
        "KLE leaf selector does not reach public EguiLanguageEditor::show through RawInput"
    ));
}

#[test]
fn rejects_leaf_without_kle_raw_input_construction() {
    let fixture = Fixture::new();
    let mut leaf = FIXTURE_LEAF;
    leaf.kle.harness.raw_input_construction.marker = "missing raw input construction";
    leaf.kle.harness.raw_input_construction.line = 1;
    let error = validate_leaf(&leaf, &fixture.katana_root, &fixture.kle_root).unwrap_err();
    assert!(error.contains(
        "KLE leaf selector does not reach public EguiLanguageEditor::show through RawInput"
    ));
}

#[test]
fn rejects_leaf_without_public_show_raw_input_harness_callsite() {
    let fixture = Fixture::new();
    let mut leaf = FIXTURE_LEAF;
    leaf.kle.harness.public_show_callsite.marker = "result = Some(editor.render(ui))";
    leaf.kle.harness.public_show_callsite.line = 1;
    assert!(
        validate_leaf(&leaf, &fixture.katana_root, &fixture.kle_root)
            .unwrap_err()
            .contains("KLE public show RawInput harness does not invoke editor.show(ui)"),
    );
}

#[test]
fn rejects_leaf_without_exact_leaf_id_assertion() {
    let fixture = Fixture::new();
    fs::write(
        fixture.kle_root.join("tests/leaf.rs"),
        fixture_leaf_test_source(false, true, true),
    )
    .unwrap();
    assert!(
        validate_leaf(&FIXTURE_LEAF, &fixture.katana_root, &fixture.kle_root)
            .unwrap_err()
            .contains("does not enumerate toolbar.bold via assert_leaf_case")
    );
}

#[test]
fn rejects_leaf_without_category_artifact_or_accesskit_assertions() {
    let fixture = Fixture::new();
    fs::write(
        fixture.kle_root.join("tests/leaf.rs"),
        fixture_leaf_test_source(true, false, true),
    )
    .unwrap();
    assert!(
        validate_leaf(&FIXTURE_LEAF, &fixture.katana_root, &fixture.kle_root)
            .unwrap_err()
            .contains("assert_toolbar_artifact_leaf")
    );

    fs::write(
        fixture.kle_root.join("tests/leaf.rs"),
        fixture_leaf_test_source(true, true, false),
    )
    .unwrap();
    assert!(
        validate_leaf(&FIXTURE_LEAF, &fixture.katana_root, &fixture.kle_root)
            .unwrap_err()
            .contains("assert_toolbar_accesskit_leaf")
    );
}

#[test]
fn rejects_missing_actual_host_effect_evidence_without_hiding_kle_validation() {
    let fixture = Fixture::new();
    let mut leaf = FIXTURE_LEAF;
    leaf.host_e2e.effect = HostEffectKind::Missing;
    let error = validate_leaf(&leaf, &fixture.katana_root, &fixture.kle_root).unwrap_err();
    assert!(error.contains("missing actual KLE host-effect E2E coverage"));
    assert!(!error.contains("does not reach public EguiLanguageEditor::show through RawInput"));
}

#[test]
fn collects_source_kle_and_host_failures_for_one_leaf() {
    let fixture = Fixture::new();
    let mut leaf = FIXTURE_LEAF;
    leaf.source.marker = "missing-marker";
    leaf.kle.source_path = "tests/missing.rs";
    leaf.host_e2e.effect = HostEffectKind::Missing;
    let error = validate_leaf(&leaf, &fixture.katana_root, &fixture.kle_root).unwrap_err();
    assert!(error.contains("leaf source marker mismatch"));
    assert!(error.contains("missing KLE leaf actual-input source"));
    assert!(error.contains("missing actual KLE host-effect E2E coverage"));
}
