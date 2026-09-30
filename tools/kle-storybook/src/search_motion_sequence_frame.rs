use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(super) struct MotionArtifactFrame {
    pub(super) stage_id: String,
    pub(super) scenario_stage: String,
    pub(super) event_cardinality: usize,
    pub(super) artifact_receipt: katana_ui_core::egui::OpaqueRootArtifactReceipt,
    pub(super) receipt_record_hash: String,
}

impl MotionArtifactFrame {
    pub(super) fn from_root(
        root: crate::host_types::StorybookRootFrame,
        stage_id: &str,
        scenario_stage: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            stage_id: stage_id.to_string(),
            scenario_stage,
            event_cardinality: root.receipt.event_cardinality(),
            artifact_receipt: root.artifact_receipt,
            receipt_record_hash: root.receipt.record_hash().to_string(),
        })
    }
}

pub(super) fn prepare_output(output_dir: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(output_dir)?;
    Ok(output_dir.to_path_buf())
}

pub(super) fn capture_stage_id(index: usize) -> String {
    format!("frame-{index:03}")
}
