use std::{fs, path::Path};

use super::context_menu_contract_types::ContextMenuContractLeaf;

#[path = "context_menu_contract_gate.rs"]
mod context_menu_contract_gate;
#[path = "context_menu_contract_source_scan.rs"]
mod context_menu_contract_source_scan;

pub(super) struct ContextMenuHostE2eValidator;

impl ContextMenuHostE2eValidator {
    pub(super) fn validate(leaves: &[ContextMenuContractLeaf], root: &Path) -> Result<(), String> {
        let test_source = Self::read_leaf_source(leaves, root, true)?;
        let runtime_source = Self::read_leaf_source(leaves, root, false)?;
        Self::validate_test_scope(&test_source)?;
        Self::validate_runtime_scope(&runtime_source)?;
        context_menu_contract_gate::validate_just_gate(root)
    }

    fn read_leaf_source(
        leaves: &[ContextMenuContractLeaf],
        root: &Path,
        is_test: bool,
    ) -> Result<String, String> {
        let leaf = leaves
            .first()
            .ok_or_else(|| "context-menu structural inventory is empty".to_string())?;
        let path = if is_test {
            leaf.host_e2e_test_source_path
        } else {
            leaf.host_e2e_runtime_source_path
        };
        let kind = if is_test { "test" } else { "runtime" };
        fs::read_to_string(root.join(path))
            .map_err(|_| format!("missing ContextMenu host E2E {kind} source: {path}"))
    }

    fn validate_test_scope(source: &str) -> Result<(), String> {
        Self::reject_prohibited_source(source, "ContextMenu host E2E test")?;
        let code = context_menu_contract_source_scan::code_only(source);
        Self::require_markers(
            &code,
            &[
                ("fixed host dependency", "use katana_host_e2e_fixed::{"),
                (
                    "fixed source harness request",
                    "FixedSourceHarnessRequest {",
                ),
                ("fixed source harness build", ".build()"),
                ("fixed source metadata resolution", ".resolve_metadata()"),
                ("fixed host session", "FixedHostSession::new("),
                ("fixed host initial frame", ".initial_frame()"),
                (
                    "fixed host current-frame route",
                    ".current_frame_for_target(",
                ),
                ("executable test", "#[test]"),
                (
                    "all-action case inventory",
                    "ContextMenuSourceInventory::case_specs()",
                ),
                (
                    "all-route iteration",
                    "for route in ContextMenuOpenRoute::ALL",
                ),
                (
                    "all-route cardinality",
                    "assert_eq!(ContextMenuOpenRoute::ALL.len(), 3)",
                ),
                (
                    "physical request generation",
                    "request_for_spec(route, spec)",
                ),
                (
                    "root order and Format test",
                    "ContextMenuSourceInventory::assert_root_order_and_format_visibility(&audit)",
                ),
                ("Format absence test", "format_is_absent_for_non_markdown()"),
                ("authoring inventory test", "assert_eq!(specs.len(), 30)"),
                (
                    "13 plus 17 operation count",
                    "assert_eq!(direct_authoring_count + code_kind_count, 30)",
                ),
                (
                    "host effect observation",
                    "assert_ne!(before_host_state, after_host_state)",
                ),
                (
                    "no-mutation observation",
                    "assert_eq!(before_host_state, after_host_state)",
                ),
                ("one-shot result", "assert!(first_forward.is_ok())"),
                ("replay result", "assert!(replay.is_err())"),
            ],
            "ContextMenu host E2E test",
        )
    }

    fn validate_runtime_scope(source: &str) -> Result<(), String> {
        Self::reject_prohibited_source(source, "ContextMenu host E2E runtime")?;
        let code = context_menu_contract_source_scan::code_only(source);
        Self::require_markers(
            &code,
            &[
                (
                    "actual KLE editor",
                    "use katana_language_editor_egui::{EguiTextCommandSurfaceEditor, KucRootBindingReceipt};",
                ),
                (
                    "KUC frame record",
                    "context_menu::{EguiContextMenuFrameRecord, EguiContextMenuItemFrame}",
                ),
                (
                    "public editor construction",
                    "EguiTextCommandSurfaceEditor::new(",
                ),
                (
                    "public opaque editor route",
                    "editor.show_with_opaque_frame(",
                ),
                (
                    "physical KLE input request",
                    "request.apply_to_raw_input_once(&mut input)",
                ),
                ("KUC receipt one-shot state", "receipt.consumed_once()"),
                ("current-frame locator request", "locator.request("),
                (
                    "refreshed record loop",
                    "for refresh in 0..=MAX_RECORD_REFRESHES",
                ),
                (
                    "visible-record bounds check",
                    "fn assert_visible_record_bounds(",
                ),
                (
                    "no offscreen public record",
                    "bounds_contains(record.bounds, item.bounds)",
                ),
                (
                    "physical item press",
                    "primary_click_bounds(context, editor, item.bounds)",
                ),
                ("no mutation after replay", "replay.is_err() && unchanged"),
            ],
            "ContextMenu host E2E runtime",
        )
    }

    fn require_markers(
        source: &str,
        requirements: &[(&str, &str)],
        scope: &str,
    ) -> Result<(), String> {
        for &(description, marker) in requirements {
            if !source.contains(marker) {
                return Err(format!("{scope} is missing {description}: {marker:?}"));
            }
        }
        Ok(())
    }

    fn reject_prohibited_source(source: &str, scope: &str) -> Result<(), String> {
        let code = context_menu_contract_source_scan::code_only(source);
        const PROHIBITED: &[(&str, &str)] = &[
            ("direct AppAction injection", "AppAction::"),
            ("pending action injection", "pending_action"),
            ("trigger_action injection", "trigger_action"),
            ("mutable KatanA state setup", "app_state_mut("),
            ("direct document setup", "Document::new("),
            ("direct document list setup", "open_documents"),
            ("direct active document setup", "active_doc_idx"),
            ("fixed sleep", "thread::sleep("),
            ("fixed sleep", "Duration::from_millis("),
            ("fixed sleep", "wait_for(Duration"),
            ("KLE raw-input construction", "kle_raw_input"),
            ("KLE raw-input construction", "build_kle_raw_input("),
            (
                "KLE raw-input construction",
                "katana_language_editor::egui::RawInput",
            ),
            ("KLE coordinate map", "coordinate_map"),
            ("KLE semantic action map", "semantic_action_map"),
            ("simulator counter", "simulator_counter"),
            ("simulator counter", "fake_action_counter"),
            ("simulator evidence", "simulator_evidence"),
            ("test-only action counter", "action_counter"),
            ("test-only effect counter", "effect_counter"),
        ];
        for &(description, marker) in PROHIBITED {
            if code.contains(marker) {
                return Err(format!(
                    "{scope} contains prohibited {description}: {marker:?}"
                ));
            }
        }
        Ok(())
    }
}
