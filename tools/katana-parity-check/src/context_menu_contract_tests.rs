use super::{
    context_menu_contract_entries::ContextMenuContractEntries,
    context_menu_contract_types::ContextEvidenceKind,
    context_menu_contract_validation::ContextMenuContractAudit,
};

#[test]
fn context_menu_contract_has_exact_structural_counts() -> Result<(), String> {
    ContextMenuContractAudit::validate_declarations(&ContextMenuContractEntries::leaves())
        .map_err(|error| format!("complete context-menu contract was rejected: {error}"))
}

#[test]
fn rejects_bad_root_order() -> Result<(), String> {
    let mut leaves = ContextMenuContractEntries::leaves();
    leaves[0].root_slot = Some(1);
    assert_rejected(
        ContextMenuContractAudit::validate_declarations(&leaves),
        "root order",
    )
}

#[test]
fn rejects_duplicate_structural_leaf() -> Result<(), String> {
    let mut leaves = ContextMenuContractEntries::leaves();
    leaves[1].id = leaves[0].id;
    assert_rejected(
        ContextMenuContractAudit::validate_declarations(&leaves),
        "duplicate",
    )
}

#[test]
fn rejects_source_only_structural_leaf() -> Result<(), String> {
    let mut leaves = ContextMenuContractEntries::leaves();
    leaves[0].evidence_kind = ContextEvidenceKind::SourceOnly;
    assert_rejected(
        ContextMenuContractAudit::validate_declarations(&leaves),
        "non-actual",
    )
}

#[test]
fn rejects_every_non_actual_structural_evidence_kind() -> Result<(), String> {
    for kind in [
        ContextEvidenceKind::SourceOnly,
        ContextEvidenceKind::StorybookOnly,
        ContextEvidenceKind::SimulatorOnly,
    ] {
        let mut leaves = ContextMenuContractEntries::leaves();
        leaves[0].evidence_kind = kind;
        assert_rejected(
            ContextMenuContractAudit::validate_declarations(&leaves),
            "non-actual",
        )?;
    }
    Ok(())
}

#[test]
fn rejects_missing_parent_path() -> Result<(), String> {
    let mut leaves = ContextMenuContractEntries::leaves();
    leaves[0].parent_path = "";
    assert_rejected(
        ContextMenuContractAudit::validate_declarations(&leaves),
        "incomplete",
    )
}

#[test]
fn rejects_wrong_parent_path() -> Result<(), String> {
    let mut leaves = ContextMenuContractEntries::leaves();
    let Some(bold) = leaves
        .iter_mut()
        .find(|leaf| leaf.id == "context.structure.edit.bold")
    else {
        return Err("bold structural leaf must exist in the fixture".to_string());
    };
    bold.parent_path = "root/Ingest";
    assert_rejected(
        ContextMenuContractAudit::validate_declarations(&leaves),
        "parent path mismatch",
    )
}

#[test]
fn rejects_missing_edit_parent_path_leaf() -> Result<(), String> {
    let mut leaves = ContextMenuContractEntries::leaves();
    leaves.retain(|leaf| leaf.id != "context.structure.edit.parent-path");
    assert_rejected(
        ContextMenuContractAudit::validate_declarations(&leaves),
        "parent path",
    )
}

#[test]
fn rejects_missing_direct_markdown_leaf() -> Result<(), String> {
    let mut leaves = ContextMenuContractEntries::leaves();
    leaves.retain(|leaf| leaf.id != "context.structure.edit.bold");
    assert_rejected(
        ContextMenuContractAudit::validate_declarations(&leaves),
        "13 direct",
    )
}

#[test]
fn rejects_missing_nested_code_leaf() -> Result<(), String> {
    let mut leaves = ContextMenuContractEntries::leaves();
    leaves.retain(|leaf| leaf.id != "context.structure.code.sql");
    assert_rejected(
        ContextMenuContractAudit::validate_declarations(&leaves),
        "17 nested",
    )
}

#[test]
fn rejects_missing_visible_ingest_leaf() -> Result<(), String> {
    let mut leaves = ContextMenuContractEntries::leaves();
    leaves.retain(|leaf| leaf.id != "context.structure.ingest.file");
    assert_rejected(
        ContextMenuContractAudit::validate_declarations(&leaves),
        "2 visible ingest",
    )
}

#[test]
fn rejects_missing_actual_input_route() -> Result<(), String> {
    let mut leaves = ContextMenuContractEntries::leaves();
    leaves.retain(|leaf| leaf.id != "context.structure.route.accesskit");
    assert_rejected(
        ContextMenuContractAudit::validate_declarations(&leaves),
        "secondary, Shift+F10, and AccessKit",
    )
}

#[test]
fn rejects_missing_overflow_invariant() -> Result<(), String> {
    let mut leaves = ContextMenuContractEntries::leaves();
    leaves.retain(|leaf| leaf.id != "context.structure.overflow.visible-record-only");
    assert_rejected(
        ContextMenuContractAudit::validate_declarations(&leaves),
        "overflow invariant",
    )
}

#[test]
fn rejects_unexpected_structural_leaf() -> Result<(), String> {
    let mut leaves = ContextMenuContractEntries::leaves();
    leaves[0].id = "context.structure.root.unexpected";
    assert_rejected(
        ContextMenuContractAudit::validate_declarations(&leaves),
        "required inventory",
    )
}

#[test]
fn rejects_non_host_e2e_actual_input_path() -> Result<(), String> {
    let mut leaves = ContextMenuContractEntries::leaves();
    leaves[0].host_e2e_test_source_path =
        "crates/katana-language-editor-egui/src/context_menu_actual_input_tests.rs";
    assert_rejected(
        ContextMenuContractAudit::validate_declarations(&leaves),
        "must require the ContextMenu host E2E",
    )
}

#[test]
fn rejects_non_host_e2e_runtime_source_path() -> Result<(), String> {
    let mut leaves = ContextMenuContractEntries::leaves();
    leaves[0].host_e2e_runtime_source_path =
        "crates/katana-language-editor-egui/src/context_menu_actual_input_tests.rs";
    assert_rejected(
        ContextMenuContractAudit::validate_declarations(&leaves),
        "must require the ContextMenu host E2E",
    )
}

#[test]
fn rejects_katana_source_marker_mismatch() -> Result<(), String> {
    let mut leaves = ContextMenuContractEntries::leaves();
    leaves[0].source.marker = "missing KatanA marker";
    let katana_root = crate::source_inventory_repo::SourceInventoryRepo::resolve_katana_repo()
        .map_err(|error| format!("KatanA reference must resolve: {error}"))?;
    assert_rejected(
        ContextMenuContractAudit::validate_source(&leaves[0], &katana_root),
        "source marker mismatch",
    )
}

fn assert_rejected(result: Result<(), String>, expected: &str) -> Result<(), String> {
    match result {
        Err(error) if error.contains(expected) => Ok(()),
        Ok(()) => Err(format!("expected rejection containing {expected:?}")),
        Err(error) => Err(format!("unexpected rejection: {error}")),
    }
}
