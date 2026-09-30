#[cfg(test)]
#[path = "baseline_inventory.rs"]
mod inventory;
#[cfg(test)]
#[path = "baseline_reference.rs"]
mod reference;

use crate::baseline_evidence::{
    REFERENCE_ONLY_CONSTRAINTS, RUNNABLE_BASELINE_COMMANDS, SOURCE_OF_TRUTH_PATHS,
    SOURCE_ONLY_EDITOR_MODULES,
};
use crate::requirements::RequirementsDocumentAudit;

pub(crate) use crate::baseline_evidence::AUDIT_DOCUMENT;

pub(crate) struct KatanaBaselineAudit;

impl KatanaBaselineAudit {
    pub(crate) fn validate() -> Result<(), String> {
        RequirementsDocumentAudit::validate()?;
        validate_contains_all(SOURCE_OF_TRUTH_PATHS, "source-of-truth path")?;
        validate_contains_all(
            RUNNABLE_BASELINE_COMMANDS,
            "runnable KatanA baseline command",
        )?;
        validate_contains_all(
            SOURCE_ONLY_EDITOR_MODULES,
            "source-only KatanA editor module",
        )?;
        validate_contains_all(
            REFERENCE_ONLY_CONSTRAINTS,
            "KatanA reference-only constraint",
        )?;
        Ok(())
    }
}

fn validate_contains_all(needles: &[&str], label: &str) -> Result<(), String> {
    for needle in needles {
        if !AUDIT_DOCUMENT.contains(needle) {
            return Err(format!("audit document is missing {label}: {needle}"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_document_records_runnable_katana_baseline_commands() -> Result<(), String> {
        validate_contains_all(
            RUNNABLE_BASELINE_COMMANDS,
            "runnable KatanA baseline command",
        )
    }

    #[test]
    fn audit_document_records_source_of_truth_paths() -> Result<(), String> {
        validate_contains_all(SOURCE_OF_TRUTH_PATHS, "source-of-truth path")
    }

    #[test]
    fn audit_document_records_source_only_editor_modules() -> Result<(), String> {
        validate_contains_all(
            SOURCE_ONLY_EDITOR_MODULES,
            "source-only KatanA editor module",
        )
    }

    #[test]
    fn audit_document_records_reference_only_constraint() -> Result<(), String> {
        validate_contains_all(
            REFERENCE_ONLY_CONSTRAINTS,
            "KatanA reference-only constraint",
        )
    }
}
