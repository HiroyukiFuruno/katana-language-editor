use crate::{
    audit_evidence::KatanaAdapterEvidenceAudit, audit_runner::KatanaRunnableAudit,
    config::KatanaRepoAdapterCheckConfig, repo_paths::KatanaRepoAdapterPaths,
};

const BASELINE_TEST: &str = "editor_ui::test_integration_editor_line_numbers_visibility";
const DOWNSTREAM_TEST: &str =
    "editor_kle_downstream_adapter::kle_event_action_stream_updates_real_katana_editor_state";

pub(crate) struct KatanaRepoAdapterAudit {
    paths: KatanaRepoAdapterPaths,
}

impl KatanaRepoAdapterAudit {
    pub(crate) fn new(config: KatanaRepoAdapterCheckConfig) -> Self {
        Self {
            paths: KatanaRepoAdapterPaths::new(config),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        self.paths.validate_root()?;
        KatanaAdapterEvidenceAudit::new(&self.paths).validate()?;
        KatanaRunnableAudit::new(self.paths.root()).validate(&[BASELINE_TEST, DOWNSTREAM_TEST])
    }
}
