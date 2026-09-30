use std::path::Path;

use crate::{
    capability_manifest::KatanaSourceEvidence, source_inventory_repo::SourceInventoryRepo,
};

pub(crate) struct KatanaEvidenceValidator;

impl KatanaEvidenceValidator {
    pub(crate) fn validate_source(
        repo_root: &Path,
        evidence: &KatanaSourceEvidence,
    ) -> Result<(), String> {
        let path = repo_root.join(evidence.path);
        let source = SourceInventoryRepo::read_file(&path)
            .map_err(|_| format!("missing KatanA source: {}", evidence.path))?;
        Self::validate_source_marker(&source, evidence)
    }

    fn validate_source_marker(source: &str, evidence: &KatanaSourceEvidence) -> Result<(), String> {
        let Some(actual_line) = source.lines().nth(evidence.line.saturating_sub(1)) else {
            return Err(format!(
                "missing KatanA source line: {}:{}",
                evidence.path, evidence.line
            ));
        };
        actual_line
            .contains(evidence.marker)
            .then_some(())
            .ok_or_else(|| {
                format!(
                    "KatanA source marker mismatch: {}:{} expected {:?}",
                    evidence.path, evidence.line, evidence.marker,
                )
            })
    }
}
