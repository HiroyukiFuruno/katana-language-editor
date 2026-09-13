use super::*;

#[test]
fn blocked_status_requires_concrete_unimplemented_evidence() {
    assert!(has_incomplete_status(
        "Blocked. KUC adapter is unimplemented."
    ));
    assert!(has_explicit_blocker(
        "Blocked. KUC adapter is unimplemented."
    ));
    assert!(!has_explicit_blocker("Blocked. Follow-up required."));
}

#[test]
fn audit_document_classifies_all_feature_inventory_rows() -> Result<(), String> {
    validate_feature_inventory_rows()
}

#[test]
fn reference_only_adapter_blocker_cannot_be_promoted_by_matrix_metadata() {
    assert!(is_reference_only_adapter_blocker(
        "Blocked. The adapter path is planned only; no actual KatanA/KLE runnable evidence exists."
    ));
}
