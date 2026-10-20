use crate::capability_manifest::{HostEffectKind, KleHostE2eEvidence, KleHostE2eTestLocator};

const HOST_E2E_SOURCE: &str = "tools/katana-host-e2e/tests/actual_host.rs";

pub(super) const CONTEXT_AUTHORING_HOST_E2E: KleHostE2eEvidence = host_e2e(
    "actual_kle_context_menu_authoring_raw_input_changes_katana_document",
    HostEffectKind::ContextAuthoring,
);
pub(super) const TOOLBAR_AUTHORING_HOST_E2E: KleHostE2eEvidence = host_e2e(
    "actual_kle_command_chrome_raw_input_changes_katana_document",
    HostEffectKind::ToolbarAuthoring,
);
pub(super) const SAVE_HOST_E2E: KleHostE2eEvidence = host_e2e(
    "actual_kle_context_menu_save_request_persists_and_clears_dirty_state",
    HostEffectKind::Save,
);
pub(super) const FORMAT_HOST_E2E: KleHostE2eEvidence = host_e2e(
    "actual_kle_context_menu_format_request_changes_actual_katana_document",
    HostEffectKind::Format,
);
pub(super) const EXTERNAL_INGEST_HOST_E2E: KleHostE2eEvidence = host_e2e(
    "unavailable_external_ingest_and_clipboard_intents_stop_before_katana_mutation",
    HostEffectKind::ExternalIngestFailClosed,
);
pub(super) const MISSING_HOST_E2E: KleHostE2eEvidence = host_e2e(
    "missing_actual_kle_clipboard_file_url_host_effect",
    HostEffectKind::Missing,
);

const fn host_e2e(selector: &'static str, effect: HostEffectKind) -> KleHostE2eEvidence {
    KleHostE2eEvidence {
        test: KleHostE2eTestLocator {
            target: "actual_host",
            source_path: HOST_E2E_SOURCE,
            selector,
        },
        effect,
    }
}
