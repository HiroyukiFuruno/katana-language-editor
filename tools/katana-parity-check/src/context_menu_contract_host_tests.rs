use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use super::{
    context_menu_contract_entries::ContextMenuContractEntries,
    context_menu_contract_validation::ContextMenuContractAudit,
};

static NEXT_HOST_E2E_FIXTURE: AtomicUsize = AtomicUsize::new(0);

struct HostE2eFixture {
    root: PathBuf,
}

impl HostE2eFixture {
    fn new() -> Result<Self, String> {
        let sequence = NEXT_HOST_E2E_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "katana-parity-context-menu-host-e2e-{}-{sequence}",
            std::process::id()
        ));
        let fixture = Self { root };
        fixture.write_valid()?;
        Ok(fixture)
    }

    fn write_valid(&self) -> Result<(), String> {
        write_fixture(
            &self
                .root
                .join("tools/katana-host-e2e/tests/context_menu_input.rs"),
            host_test_source(),
        )?;
        write_fixture(
            &self
                .root
                .join("tools/katana-host-e2e/src/context_menu_input.rs"),
            host_runtime_source(),
        )?;
        write_fixture(&self.root.join("Justfile"), justfile_source())?;
        Ok(())
    }
}

impl Drop for HostE2eFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn accepts_complete_host_e2e_scope_fixture() -> Result<(), String> {
    let fixture = HostE2eFixture::new()?;
    ContextMenuContractAudit::validate_actual_input_contract_at(
        &ContextMenuContractEntries::leaves(),
        &fixture.root,
    )
    .map_err(|error| format!("complete host fixture was rejected: {error}"))
}

#[test]
fn rejects_host_e2e_scope_without_overflow_contract() -> Result<(), String> {
    let fixture = HostE2eFixture::new()?;
    write_fixture(
        &fixture
            .root
            .join("tools/katana-host-e2e/src/context_menu_input.rs"),
        "use katana_language_editor_egui::EguiLanguageEditor;\n",
    )?;
    let result = ContextMenuContractAudit::validate_actual_input_contract_at(
        &ContextMenuContractEntries::leaves(),
        &fixture.root,
    );
    assert_rejected(result, "actual KLE editor")
}

#[test]
fn rejects_host_e2e_scope_without_three_routes() -> Result<(), String> {
    let fixture = HostE2eFixture::new()?;
    let source = host_test_source().replace(
        "assert_eq!(ContextMenuOpenRoute::ALL.len(), 3)",
        "assert_eq!(ContextMenuOpenRoute::ALL.len(), 2)",
    );
    write_fixture(
        &fixture
            .root
            .join("tools/katana-host-e2e/tests/context_menu_input.rs"),
        &source,
    )?;
    let result = ContextMenuContractAudit::validate_actual_input_contract_at(
        &ContextMenuContractEntries::leaves(),
        &fixture.root,
    );
    assert_rejected(result, "all-route cardinality")
}

#[test]
fn rejects_source_only_markers_in_comments_and_strings() -> Result<(), String> {
    let fixture = HostE2eFixture::new()?;
    write_fixture(
        &fixture
            .root
            .join("tools/katana-host-e2e/tests/context_menu_input.rs"),
        &host_test_source()
            .replace("FixedSourceHarnessBuilder", "// FixedSourceHarnessBuilder")
            .replace("KatanaApp", "\"KatanaApp\"")
            .replace(
                "ContextMenuSourceInventory::case_specs()",
                "\"ContextMenuSourceInventory::case_specs()\"",
            ),
    )?;
    let result = ContextMenuContractAudit::validate_actual_input_contract_at(
        &ContextMenuContractEntries::leaves(),
        &fixture.root,
    );
    assert_rejected(result, "fixed source harness")
}

#[test]
fn rejects_direct_katana_action_injection() -> Result<(), String> {
    for (marker, message) in [
        ("AppAction::Save;", "direct AppAction injection"),
        ("pending_action = Some(action);", "pending action injection"),
        ("trigger_action(action);", "trigger_action injection"),
    ] {
        let fixture = HostE2eFixture::new()?;
        write_fixture(
            &fixture
                .root
                .join("tools/katana-host-e2e/tests/context_menu_input.rs"),
            &host_test_source().replace(
                "fn physical_context_menu_input_emits",
                &format!("{marker}\nfn physical_context_menu_input_emits"),
            ),
        )?;
        let result = ContextMenuContractAudit::validate_actual_input_contract_at(
            &ContextMenuContractEntries::leaves(),
            &fixture.root,
        );
        assert_rejected(result, message)?;
    }
    Ok(())
}

#[test]
fn rejects_direct_state_setup_and_fixed_sleep() -> Result<(), String> {
    for (marker, message) in [
        ("app_state_mut();", "mutable KatanA state setup"),
        ("Document::new();", "direct document setup"),
        (
            "open_documents.push(document);",
            "direct document list setup",
        ),
        ("active_doc_idx = Some(0);", "direct active document setup"),
        ("thread::sleep();", "fixed sleep"),
    ] {
        let fixture = HostE2eFixture::new()?;
        write_fixture(
            &fixture
                .root
                .join("tools/katana-host-e2e/src/context_menu_input.rs"),
            &host_runtime_source().replace(
                "fn run_public_kle_route()",
                &format!("{marker}\nfn run_public_kle_route()"),
            ),
        )?;
        let result = ContextMenuContractAudit::validate_actual_input_contract_at(
            &ContextMenuContractEntries::leaves(),
            &fixture.root,
        );
        assert_rejected(result, message)?;
    }
    Ok(())
}

#[test]
fn rejects_kle_owned_bypass_and_simulator_counter() -> Result<(), String> {
    for (marker, message) in [
        ("coordinate_map();", "KLE coordinate map"),
        ("semantic_action_map();", "KLE semantic action map"),
        ("kle_raw_input();", "KLE raw-input construction"),
        ("build_kle_raw_input();", "KLE raw-input construction"),
        ("simulator_counter += 1;", "simulator counter"),
        ("effect_counter += 1;", "test-only effect counter"),
    ] {
        let fixture = HostE2eFixture::new()?;
        write_fixture(
            &fixture
                .root
                .join("tools/katana-host-e2e/tests/context_menu_input.rs"),
            &host_test_source().replace(
                "fn physical_context_menu_input_emits",
                &format!("{marker}\nfn physical_context_menu_input_emits"),
            ),
        )?;
        let result = ContextMenuContractAudit::validate_actual_input_contract_at(
            &ContextMenuContractEntries::leaves(),
            &fixture.root,
        );
        assert_rejected(result, message)?;
    }
    Ok(())
}

#[test]
fn rejects_justfile_without_host_e2e_prerequisite() -> Result<(), String> {
    let fixture = HostE2eFixture::new()?;
    write_fixture(
        &fixture.root.join("Justfile"),
        "katana-host-e2e-context-menu:\n    cargo test --manifest-path tools/katana-host-e2e/Cargo.toml --locked --test context_menu_input\nkatana-parity-check:\n    cargo run\n",
    )?;
    let result = ContextMenuContractAudit::validate_actual_input_contract_at(
        &ContextMenuContractEntries::leaves(),
        &fixture.root,
    );
    assert_rejected(result, "must run ContextMenu host E2E")
}

fn write_fixture(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("create fixture directory: {error}"))?;
    }
    fs::write(path, contents).map_err(|error| format!("write fixture: {error}"))
}

fn assert_rejected(result: Result<(), String>, expected: &str) -> Result<(), String> {
    match result {
        Err(error) if error.contains(expected) => Ok(()),
        Ok(()) => Err(format!("expected rejection containing {expected:?}")),
        Err(error) => Err(format!("unexpected rejection: {error}")),
    }
}

fn host_test_source() -> &'static str {
    "use katana_host_e2e_fixed::{ContextMenuOpenRoute, ContextMenuSourceInventory, FixedHostSession, FixedSourceHarnessBuilder, FixedSourceHarnessRequest};\n#[test]\nfn physical_context_menu_input_emits_each_action_via_three_routes() -> Result<(), Error> { let harness = FixedSourceHarnessBuilder::new(FixedSourceHarnessRequest { request }); let harness = harness.build()?; let _metadata = harness.resolve_metadata()?; let mut session = FixedHostSession::new(harness.sandbox_path()); let _initial = session.initial_frame()?; let specs = ContextMenuSourceInventory::case_specs(); assert_eq!(ContextMenuOpenRoute::ALL.len(), 3); for route in ContextMenuOpenRoute::ALL { for spec in &specs { let request = request_for_spec(route, spec); let _frame = session.current_frame_for_target(&request)?; let before_host_state = host_state_before(); let first_forward = request.forward_once(); let replay = request.forward_once(); let after_host_state = host_state_after(); assert!(first_forward.is_ok()); assert!(replay.is_err()); assert_ne!(before_host_state, after_host_state); } } Ok(()) }\n#[test]\nfn physical_context_menu_input_preserves_katana_root_hierarchy() -> Result<(), Error> { let audit = ContextMenuSourceInventory::load(); ContextMenuSourceInventory::assert_root_order_and_format_visibility(&audit)?; Ok(()) }\n#[test]\nfn physical_context_menu_input_format_is_disabled_without_markdown_source() { assert!(ContextMenuSourceInventory::format_is_absent_for_non_markdown()); }\n#[test]\nfn physical_context_menu_input_keeps_katana_authoring_operation_table() { let specs = ContextMenuSourceInventory::case_specs(); let direct_authoring_count = 13; let code_kind_count = 17; let before_host_state = host_state_before(); let after_host_state = host_state_after(); assert_eq!(specs.len(), 30); assert_eq!(direct_authoring_count + code_kind_count, 30); assert_eq!(before_host_state, after_host_state); }\n"
}

fn host_runtime_source() -> &'static str {
    "use katana_language_editor_egui::{EguiTextCommandSurfaceEditor, KucRootBindingReceipt};\nuse katana_ui_core::egui::context_menu::{EguiContextMenuFrameRecord, EguiContextMenuItemFrame};\nfn run_public_kle_route() -> Result<(), Error> { let mut editor = EguiTextCommandSurfaceEditor::new(provider)?; request.apply_to_raw_input_once(&mut input)?; let receipt: KucRootBindingReceipt = editor.show_with_opaque_frame(ui, &mut forwarder, |locator| { locator.request(selector)?; Ok(()) })?; assert!(receipt.consumed_once()); Ok(()) }\nfn reveal_current_frame_item() { for refresh in 0..=MAX_RECORD_REFRESHES { primary_click_bounds(context, editor, item.bounds); } }\nfn assert_visible_record_bounds() { bounds_contains(record.bounds, item.bounds); }\nfn replay_guard() { let replay = request.apply_to_raw_input_once(&mut input); let unchanged = replay.is_err(); assert!(replay.is_err() && unchanged); }\n"
}

fn justfile_source() -> &'static str {
    "katana-host-e2e-context-menu:\n    cargo test --manifest-path tools/katana-host-e2e/Cargo.toml --locked --test context_menu_input\nkatana-parity-check:\n    just katana-host-e2e-context-menu\n"
}
