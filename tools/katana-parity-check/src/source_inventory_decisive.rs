use std::path::Path;

use crate::{
    capability_manifest::KatanaSourceEvidence, source_inventory_repo::SourceInventoryRepo,
};

const SOURCES: &[KatanaSourceEvidence] = &[
    source(
        "crates/katana-ui/src/app/action/clipboard_image.rs",
        14,
        "pub(crate) fn has_image_payload",
    ),
    source(
        "crates/katana-ui/src/app/action/clipboard_image.rs",
        34,
        "pub(crate) fn read_image_payload",
    ),
    source(
        "crates/katana-ui/src/app/action/clipboard_file_url.rs",
        15,
        "pub(crate) fn read_image_payload",
    ),
    source(
        "crates/katana-ui/src/app/action/clipboard_image_macos.rs",
        5,
        "pub(super) fn macos_pasteboard_has_image",
    ),
    source(
        "crates/katana-ui/src/app/action/clipboard_image_macos.rs",
        31,
        "pub(super) fn read_macos_pasteboard_image",
    ),
    source(
        "crates/katana-ui/src/state/command_inventory/edit_commands.rs",
        17,
        "pub fn get",
    ),
    source(
        "crates/katana-ui/src/state/shortcut_context.rs",
        30,
        "pub fn resolve",
    ),
    source(
        "crates/katana-ui/src/state/shortcut_context.rs",
        89,
        "pub fn context_allows",
    ),
    source(
        "crates/katana-ui/src/shell_ui/shell_ui_shortcuts.rs",
        8,
        "pub(super) fn handle_shortcuts",
    ),
    source(
        "crates/katana-ui/src/shell_ui/shell_ui_shortcuts.rs",
        84,
        "fn command_shortcut_consumed",
    ),
    source(
        "crates/katana-ui/src/app/action/process_markdown_formatting.rs",
        16,
        "pub(crate) fn handle_action_format_markdown_file",
    ),
    source(
        "crates/katana-ui/src/app/action/process_markdown_formatting.rs",
        155,
        "fn refresh_after_format",
    ),
    source(
        "crates/katana-ui/src/app/action/refresh_content.rs",
        18,
        "pub(super) fn apply_refreshed_content",
    ),
    source(
        "crates/katana-ui/src/app/action/refresh_content.rs",
        108,
        "pub(super) fn handle_action_refresh_document",
    ),
    source(
        "crates/katana-ui/src/views/panels/editor/diagnostics_ui.rs",
        7,
        "pub(crate) fn render_diagnostics",
    ),
    source(
        "crates/katana-ui/src/views/panels/editor/diagnostics_popup.rs",
        10,
        "pub(crate) fn show",
    ),
    source(
        "crates/katana-ui/tests/integration/editor/ui.rs",
        1,
        "use accesskit::Role",
    ),
];

const fn source(path: &'static str, line: usize, marker: &'static str) -> KatanaSourceEvidence {
    KatanaSourceEvidence { path, line, marker }
}

pub(crate) struct DecisiveSourceInventoryAudit;

impl DecisiveSourceInventoryAudit {
    pub(crate) fn validate(repo_root: &Result<std::path::PathBuf, String>) -> Result<(), String> {
        let root = repo_root
            .as_ref()
            .map_err(|error| format!("failed to resolve KatanA reference: {error}"))?;
        validate_sources(SOURCES, root)
    }
}

fn validate_sources(sources: &[KatanaSourceEvidence], root: &Path) -> Result<(), String> {
    for source in sources {
        if source.path.is_empty() || source.line == 0 || source.marker.is_empty() {
            return Err("decisive source inventory contains an incomplete mapping".to_string());
        }
        let contents = SourceInventoryRepo::read_file(&root.join(source.path))?;
        let line = contents.lines().nth(source.line - 1).ok_or_else(|| {
            format!(
                "decisive source line is missing: {}:{}",
                source.path, source.line
            )
        })?;
        if !line.contains(source.marker) {
            return Err(format!(
                "decisive source marker mismatch: {}:{} expected {:?}",
                source.path, source.line, source.marker
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decisive_source_entrypoint_remains_test_only() {
        let _ = DecisiveSourceInventoryAudit::validate
            as fn(&Result<std::path::PathBuf, String>) -> Result<(), String>;
    }

    #[test]
    fn rejects_empty_decisive_marker() -> Result<(), String> {
        let root = std::env::temp_dir();
        let error = validate_sources(&[source("missing.rs", 1, "")], &root)
            .err()
            .ok_or_else(|| "empty decisive marker was accepted".to_string())?;
        assert!(error.contains("incomplete mapping"));
        Ok(())
    }

    #[test]
    fn rejects_missing_decisive_source() -> Result<(), String> {
        let root = std::env::temp_dir();
        let error = validate_sources(&[source("missing.rs", 1, "marker")], &root)
            .err()
            .ok_or_else(|| "missing decisive source was accepted".to_string())?;
        assert!(error.contains("source inventory file is missing"));
        Ok(())
    }
}
