use std::path::Path;

use crate::source_inventory_repo::SourceInventoryRepo;

use super::capability_manifest_host_e2e_syntax::ReachableEvidence;

const HOST_RUNTIME_SOURCE: &str = "tools/katana-host-e2e/src/host.rs";
const HOST_RUNTIME_BRIDGE: &str = "run_editor_request_with_context";
const HOST_RUNTIME_UI_FRAME: &str = "Self::run_ui_frame_with_context";
const REJECTED_SYNTHETIC_EFFECTS: &[&str] =
    &["AppAction::UpdateBuffer", "Self::update_buffer_action"];

pub(super) struct HostRuntimeRouteValidator;

impl HostRuntimeRouteValidator {
    pub(super) fn validate(workspace_root: &Path) -> Result<(), String> {
        let source = SourceInventoryRepo::read_file(&workspace_root.join(HOST_RUNTIME_SOURCE))
            .map_err(|_| format!("missing KLE host runtime: {HOST_RUNTIME_SOURCE}"))?;
        let file = syn::parse_file(&source)
            .map_err(|error| format!("invalid KLE host runtime source: {error}"))?;
        let method = Self::find_runtime_bridge(&file)?;
        let runtime = ReachableEvidence::from_block(&method.block);
        Self::validate_actual_dispatch(&runtime)?;
        Self::reject_synthetic_effects(&runtime)
    }

    fn validate_actual_dispatch(runtime: &ReachableEvidence) -> Result<(), String> {
        if !runtime.methods.contains("trigger_action") {
            return Err(
                "KatanaHost runtime bridge lacks actual KatanaApp::trigger_action route"
                    .to_string(),
            );
        }
        if !runtime.path_starts_with(HOST_RUNTIME_UI_FRAME) {
            return Err(format!(
                "KatanaHost runtime bridge lacks actual app UI-frame route: {HOST_RUNTIME_UI_FRAME}"
            ));
        }
        Ok(())
    }

    fn reject_synthetic_effects(runtime: &ReachableEvidence) -> Result<(), String> {
        if REJECTED_SYNTHETIC_EFFECTS
            .iter()
            .any(|effect| runtime.path_starts_with(effect))
        {
            return Err(
                "KatanaHost runtime bridge uses rejected synthetic UpdateBuffer host effect"
                    .to_string(),
            );
        }
        Ok(())
    }

    fn find_runtime_bridge(file: &syn::File) -> Result<&syn::ImplItemFn, String> {
        file.items
            .iter()
            .filter_map(|item| match item {
                syn::Item::Impl(implementation) => Some(implementation.items.iter()),
                _ => None,
            })
            .flatten()
            .find_map(|item| match item {
                syn::ImplItem::Fn(function) if function.sig.ident == HOST_RUNTIME_BRIDGE => {
                    Some(function)
                }
                _ => None,
            })
            .ok_or_else(|| format!("missing KatanaHost::{HOST_RUNTIME_BRIDGE} runtime bridge"))
    }
}
