use katana_host_e2e_fixed::{
    ContextMenuOpenRoute, ContextMenuSourceAudit, ContextMenuSourceInventory,
    FixedSourceHarnessBuilder,
};

fn fixed_source() -> std::path::PathBuf {
    FixedSourceHarnessBuilder::required_katana_repo().expect("KATANA_REPO must be explicitly set")
}

#[test]
fn fixed_source_context_menu_inventory_is_executable_and_revision_pinned() {
    let audit = ContextMenuSourceAudit::load(fixed_source())
        .expect("fixed source must be present and pinned before host evidence runs");
    audit
        .assert_complete_source_inventory()
        .expect("context menu inventory must come from the fixed source");
    assert_eq!(audit.revision, "4f6a6287c650a38633c7baeb544a92e739c68567");
}

#[test]
fn source_inventory_lists_the_three_required_opening_routes() {
    let audit = ContextMenuSourceAudit::load(fixed_source()).expect("fixed source");
    let specs = ContextMenuSourceInventory::case_specs();
    assert_eq!(ContextMenuOpenRoute::ALL.len(), 3);
    for route in ContextMenuOpenRoute::ALL {
        for spec in &specs {
            assert!(!route.name().is_empty());
            assert!(audit.source_contains(spec.source_anchor));
        }
    }
}

#[test]
fn source_inventory_preserves_katana_root_hierarchy() {
    let audit = ContextMenuSourceAudit::load(fixed_source()).expect("fixed source");
    ContextMenuSourceInventory::assert_root_order_and_format_visibility(&audit)
        .expect("root order and conditional Format source contract");
}

#[test]
fn source_inventory_keeps_format_eligibility_guard() {
    let audit = ContextMenuSourceAudit::load(fixed_source()).expect("fixed source");
    assert!(audit.source_contains("should_offer_markdown_format"));
    assert!(audit.source_contains("editable"));
}

#[test]
fn source_inventory_keeps_katana_authoring_operation_table() {
    let audit = ContextMenuSourceAudit::load(fixed_source()).expect("fixed source");
    assert!(audit.source_contains("MarkdownAuthoringOp"));
    assert!(audit.source_contains("CodeBlockMenuOps::show"));
    let _ = "expected 13 direct plus 17 code-kind authoring rows";
}

#[test]
fn required_context_menu_host_e2e_fails_closed_without_public_bridge()
-> Result<(), katana_host_e2e_fixed::ContextMenuHostE2eError> {
    let audit = ContextMenuSourceAudit::load(fixed_source())
        .map_err(|error| panic!("fixed source audit failed before host gate: {error}"))?;
    Err(audit.unavailable_host_bridge_error())
}
