use std::fs;

use super::super::{FIXTURE_LEAF, Fixture, validate_inventory, validate_leaf};
use crate::leaf_inventory_entries::LEAVES;

use crate::leaf_inventory_types::MISSING_LEAF_SCHEMA_FIELD;
#[test]
fn rejects_leaf_with_missing_katana_source_symbol() {
    let fixture = Fixture::new();
    let mut leaf = FIXTURE_LEAF;
    leaf.katana_source_symbol = "";
    assert!(
        validate_leaf(&leaf, &fixture.katana_root, &fixture.kle_root)
            .unwrap_err()
            .contains("missing a KatanA source symbol")
    );
}

#[test]
fn rejects_leaf_with_missing_visible_menu_path() {
    let fixture = Fixture::new();
    let mut leaf = FIXTURE_LEAF;
    leaf.visible_menu_path = "";
    assert!(
        validate_leaf(&leaf, &fixture.katana_root, &fixture.kle_root)
            .unwrap_err()
            .contains("missing a visible/menu path")
    );
}

#[test]
fn rejects_leaf_with_missing_state_condition() {
    let fixture = Fixture::new();
    let mut leaf = FIXTURE_LEAF;
    leaf.state_condition = MISSING_LEAF_SCHEMA_FIELD;
    assert!(
        validate_leaf(&leaf, &fixture.katana_root, &fixture.kle_root)
            .unwrap_err()
            .contains("missing a state condition")
    );
}

#[test]
fn rejects_leaf_with_missing_typed_kle_request_or_state() {
    let fixture = Fixture::new();
    let mut leaf = FIXTURE_LEAF;
    leaf.typed_kle_request_or_state = "";
    assert!(
        validate_leaf(&leaf, &fixture.katana_root, &fixture.kle_root)
            .unwrap_err()
            .contains("missing typed KLE request/state")
    );
}

#[test]
fn rejects_leaf_with_missing_expected_actual_katana_effect() {
    let fixture = Fixture::new();
    let mut leaf = FIXTURE_LEAF;
    leaf.expected_actual_katana_effect = "";
    assert!(
        validate_leaf(&leaf, &fixture.katana_root, &fixture.kle_root)
            .unwrap_err()
            .contains("missing expected actual KatanA effect")
    );
}

#[test]
fn rejects_top_level_group_as_leaf_id() {
    let fixture = Fixture::new();
    let mut leaf = FIXTURE_LEAF;
    leaf.id = "toolbar";
    assert!(
        validate_leaf(&leaf, &fixture.katana_root, &fixture.kle_root)
            .unwrap_err()
            .contains("top-level group aggregate")
    );
}

#[test]
fn rejects_shared_selector_between_leaves() {
    let mut leaves = LEAVES.to_vec();
    let mut second = leaves[1];
    second.id = "leaf.shared-selector";
    leaves[1] = second;
    assert!(
        validate_inventory(&leaves)
            .unwrap_err()
            .contains("shares selector")
    );
}

#[test]
fn rejects_missing_kle_leaf_source() {
    let fixture = Fixture::new();
    let mut leaf = FIXTURE_LEAF;
    leaf.kle.source_path = "tests/missing.rs";
    assert!(
        validate_leaf(&leaf, &fixture.katana_root, &fixture.kle_root)
            .unwrap_err()
            .contains("missing KLE leaf actual-input source")
    );
}

#[test]
fn rejects_missing_kle_leaf_selector() {
    let fixture = Fixture::new();
    let mut leaf = FIXTURE_LEAF;
    leaf.kle.selector = "public_show_missing";
    assert!(
        validate_leaf(&leaf, &fixture.katana_root, &fixture.kle_root)
            .unwrap_err()
            .contains("missing KLE leaf actual-input test selector")
    );
}

#[test]
fn rejects_selector_that_does_not_reach_public_show_raw_input() {
    let fixture = Fixture::new();
    fs::write(
        fixture.kle_root.join("tests/leaf.rs"),
        "#[test]\nfn public_show_leaf() { Scenario::other(); }\n",
    )
    .unwrap();
    assert!(
        validate_leaf(&FIXTURE_LEAF, &fixture.katana_root, &fixture.kle_root)
            .unwrap_err()
            .contains("does not reach public EguiLanguageEditor::show through RawInput")
    );
}
