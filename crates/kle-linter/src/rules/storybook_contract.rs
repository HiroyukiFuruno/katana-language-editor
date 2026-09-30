use crate::diagnostics::{KleLintError, Violation};
use crate::workspace::{SourceFile, WorkspaceModel};
use std::path::Path;

const STORYBOOK_ROOT: &str = "tools/kle-storybook/src";
const FORBIDDEN_PATTERNS: &[(&str, &str)] = &[
    ("manual-hit-test", "rect_contains"),
    ("manual-hit-test", "HitRect"),
    ("manual-hit-test", "manual_hit_test"),
    ("string-action-synthesis", "UiAction::SetValue"),
    ("string-action-synthesis", "parse_state_id"),
    ("string-action-synthesis", "action_from_label"),
    ("string-action-synthesis", "string_to_action"),
    ("string-action-synthesis", "settings-field:"),
    ("contract-bypass", "KleContractBypass"),
    ("contract-bypass", "KucContractBypass"),
    ("direct-kle-binding", "HostProjectionBinding"),
    ("fallback-renderer", "StorybookFallbackRenderer"),
    ("fallback-renderer", "fallback_pixels"),
    ("fallback-renderer", "MinifbFallback"),
    ("shape-count-acceptance", "shape_count"),
    ("direct-kle-show", "host.editor.show(ui)"),
    ("direct-kle-show", "editor.show(ui)"),
    ("direct-kuc-aggregate", "latest_kuc_artifact_aggregate"),
    ("direct-kuc-aggregate", "artifact_paint_plans"),
    ("direct-kuc-compositor", "ArtifactCompositor"),
    ("local-motion-codec", "gif::"),
    ("local-motion-codec", "png::"),
    ("local-motion-codec", "ffmpeg"),
    ("local-motion-codec", "GifEncoder"),
    ("local-motion-codec", "PngDecoder"),
    ("local-motion-codec", "decode_rgba"),
    ("kle-owned-text-surface", "TextSurface"),
    ("kle-owned-text-surface", "TextArea"),
    ("kle-owned-text-surface", "TextSurfaceViewport"),
    (
        "kle-owned-presentation",
        "EguiTextCommandSurfacePresentation",
    ),
    ("kle-owned-style", "TextCommandSurfaceStyle"),
    ("kle-owned-selection", "selection_start"),
    ("kle-owned-selection", "selection_end"),
    ("kle-owned-fixture", "FIXTURE_TEXT"),
    ("direct-scenario-router", "issue_with_router"),
    ("direct-root-context", "KucRootEventBatchContext"),
    ("counter-only-dispatch", "StorybookGenericEffectReceipt"),
    ("counter-only-dispatch", "StorybookOpaqueReceiptHost"),
];
const REQUIRED_CONSUMER_PATTERNS: &[&str] = &[
    "HostProjectionProvider",
    "KucRootBinding",
    "show_write_artifact",
    "EguiTextCommandSurfaceRootEventDispatchReceipt",
    "dispatch_once",
    "OpaqueRootArtifactReceipt",
    "FullTextCommandSurfaceScenarioFactory",
    "FullTextCommandSurfaceScenarioSession",
    "FullTextCommandSurfaceScenarioId",
];

pub struct StorybookContractRule;

impl StorybookContractRule {
    pub fn check(workspace: &WorkspaceModel) -> Result<Vec<Violation>, KleLintError> {
        let root = workspace.root().join(STORYBOOK_ROOT);
        let mut violations = Vec::new();
        let mut required_consumer_patterns = vec![false; REQUIRED_CONSUMER_PATTERNS.len()];
        for file in workspace.rust_files() {
            if !file.is_under(&root) || Self::is_test_source(file) {
                continue;
            }
            for (index, pattern) in REQUIRED_CONSUMER_PATTERNS.iter().enumerate() {
                required_consumer_patterns[index] |= file.source().contains(pattern);
            }
            violations.extend(Self::check_file(file));
        }
        for (index, pattern) in REQUIRED_CONSUMER_PATTERNS.iter().enumerate() {
            if !required_consumer_patterns[index] {
                violations.push(Self::violation(
                    &root.join("search_motion_sequence.rs"),
                    1,
                    "real-kuc-root-consumer",
                    pattern,
                ));
            }
        }
        Ok(violations)
    }

    fn check_file(file: &SourceFile) -> Vec<Violation> {
        let mut violations = Vec::new();
        for (rule, pattern) in FORBIDDEN_PATTERNS {
            if let Some(line) = Self::first_line(file.source(), pattern) {
                violations.push(Self::violation(file.path(), line, rule, pattern));
            }
        }
        violations
    }

    fn first_line(source: &str, pattern: &str) -> Option<usize> {
        source
            .lines()
            .enumerate()
            .find_map(|(index, line)| Self::contains_symbol(line, pattern).then_some(index + 1))
    }

    fn is_test_source(file: &SourceFile) -> bool {
        file.path()
            .file_name()
            .is_some_and(|name| name == "tests.rs" || name == "tests_host_projection.rs")
    }

    fn contains_symbol(line: &str, pattern: &str) -> bool {
        line.match_indices(pattern).any(|(index, _)| {
            let before = line[..index].chars().next_back();
            let after = line[index + pattern.len()..].chars().next();
            !before.is_some_and(Self::is_identifier_character)
                && !after.is_some_and(Self::is_identifier_character)
        })
    }

    fn is_identifier_character(character: char) -> bool {
        character == '_' || character.is_alphanumeric()
    }

    fn violation(path: &Path, line: usize, rule: &str, pattern: &str) -> Violation {
        Violation::new(
            path.to_path_buf(),
            line,
            1,
            "storybook-contract-bypass",
            format!(
                "{rule} pattern `{pattern}` must use the actual KUC/KLE Storybook surface contract."
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::StorybookContractRule;

    #[test]
    fn exact_symbol_matching_accepts_kuc_event_types_but_rejects_owned_surfaces() {
        assert!(StorybookContractRule::contains_symbol(
            "let surface = TextSurface::new();",
            "TextSurface"
        ));
        assert!(!StorybookContractRule::contains_symbol(
            "use katana_ui_core::text_surface::TextSurfaceEvent;",
            "TextSurface"
        ));
    }
}
