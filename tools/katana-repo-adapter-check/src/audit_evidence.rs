use crate::repo_paths::KatanaRepoAdapterPaths;

const MANIFEST_EVIDENCE: &[&str] = &[
    "katana-language-editor = { path = \"../../../katana-language-editor/crates/katana-language-editor\" }",
    "katana-language-editor-egui = { path = \"../../../katana-language-editor/crates/katana-language-editor-egui\" }",
];
const WRAPPER_EVIDENCE: &[&str] = &[
    "#[path = \"integration/editor/kle_downstream_adapter.rs\"]",
    "mod editor_kle_downstream_adapter;",
];
const ADAPTER_SOURCE_EVIDENCE: &[&str] = &[
    "kle_event_action_stream_updates_real_katana_editor_state",
    "KatanaKleEditorAdapter::drain_editor",
    "EguiLanguageEditor::new(editor_config())",
    "EditorAction::SaveDocument",
    "EditorAction::SetViewMode(EditorViewMode::Split)",
    "EditorAction::SetSplitDirection(EditorSplitDirection::Vertical)",
    "EditorScrollSource::Preview",
    ".with_target_line(7)",
    "ScrollSource::Neither",
    "last_scroll_to_line",
    "active_document_buffer(&harness)",
    "KatanA document should become dirty after KLE content event",
    "std::fs::read_to_string(&second_path)",
    "KatanA save action should mark the document clean",
];
const FORBIDDEN_ADAPTER_EVIDENCE: &[&str] = &[
    "force_view_mode",
    "# Updated from KLE adapter",
    "katana_language_editor::LanguageEditor::poll_events",
    "Arc<dyn SyntaxHighlighter>",
];
const COMPOSE_EVIDENCE: &[&str] =
    &["../../../../katana-language-editor:/katana-language-editor:ro"];
const KLE_WORKFLOW_CLONE_EVIDENCE: &[&str] = &[
    "KLE_REF",
    "release/v0.1.0",
    "../katana-language-editor",
    "https://github.com/HiroyukiFuruno/katana-language-editor",
];

pub(crate) struct KatanaAdapterEvidenceAudit<'a> {
    paths: &'a KatanaRepoAdapterPaths,
}

impl<'a> KatanaAdapterEvidenceAudit<'a> {
    pub(crate) fn new(paths: &'a KatanaRepoAdapterPaths) -> Self {
        Self { paths }
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        let mut missing = Vec::new();
        let manifest = self.paths.read_optional(self.paths.manifest_path())?;
        validate_document(
            "katana-ui manifest",
            manifest.as_deref(),
            MANIFEST_EVIDENCE,
            &mut missing,
        );
        let wrapper = self.paths.read_optional(self.paths.wrapper_path())?;
        validate_document(
            "ui integration wrapper",
            wrapper.as_deref(),
            WRAPPER_EVIDENCE,
            &mut missing,
        );
        let adapter = self.paths.read_optional(self.paths.adapter_path())?;
        validate_document(
            "KLE downstream adapter source",
            adapter.as_deref(),
            ADAPTER_SOURCE_EVIDENCE,
            &mut missing,
        );
        if let Some(adapter) = adapter.as_deref() {
            validate_absent(
                "KLE downstream adapter source",
                adapter,
                FORBIDDEN_ADAPTER_EVIDENCE,
                &mut missing,
            );
        }
        validate_document(
            "check-linux compose mount",
            self.paths
                .read_optional(self.paths.compose_linux_path())?
                .as_deref(),
            COMPOSE_EVIDENCE,
            &mut missing,
        );
        validate_document(
            "check-windows compose mount",
            self.paths
                .read_optional(self.paths.compose_windows_path())?
                .as_deref(),
            COMPOSE_EVIDENCE,
            &mut missing,
        );
        for (label, path) in [
            (
                "workflow test-and-build clone",
                self.paths.workflow_test_and_build_path(),
            ),
            (
                "workflow release-readiness clone",
                self.paths.workflow_release_readiness_path(),
            ),
            (
                "workflow build-and-release clone",
                self.paths.workflow_build_and_release_path(),
            ),
        ] {
            validate_document(
                label,
                self.paths.read_optional(path)?.as_deref(),
                KLE_WORKFLOW_CLONE_EVIDENCE,
                &mut missing,
            );
        }
        if missing.is_empty() {
            return Ok(());
        }
        Err(format_missing(missing))
    }
}

fn validate_document(
    label: &str,
    document: Option<&str>,
    needles: &[&str],
    missing: &mut Vec<String>,
) {
    let Some(document) = document else {
        missing.push(format!("missing {label}"));
        return;
    };
    for needle in needles {
        if !document.contains(needle) {
            missing.push(format!("{label} is missing `{needle}`"));
        }
    }
}

fn validate_absent(label: &str, document: &str, needles: &[&str], missing: &mut Vec<String>) {
    for needle in needles {
        if document.contains(needle) {
            missing.push(format!(
                "{label} contains forbidden planned-patch marker `{needle}`"
            ));
        }
    }
}

fn format_missing(missing: Vec<String>) -> String {
    format!(
        "actual KatanA downstream KLE evidence is absent; baseline/source/planned text is not live evidence:\n{}",
        missing
            .into_iter()
            .map(|item| format!("- {item}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}
