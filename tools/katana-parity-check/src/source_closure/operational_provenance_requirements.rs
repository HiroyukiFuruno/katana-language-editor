use std::fs;
use std::path::Path;

use super::operational_input::EvidenceRef;
use super::operational_output::capture_bytes;

pub(super) fn capture_aliases(
    stage_root: &Path,
    aliases: &Path,
    run_id: &str,
    evidence: &mut Vec<EvidenceRef>,
) -> Result<EvidenceRef, String> {
    let bytes = fs::read(aliases)
        .map_err(|error| format!("failed to read requirement source aliases: {error}"))?;
    super::source_requirement_alias_ledger::RequirementSourceAliasLedger::validate_captured_bytes(
        &bytes,
    )?;
    capture_bytes(
        stage_root,
        "provenance/requirements/editor-requirement-source-aliases.json",
        &bytes,
        run_id,
        "read-only-local-source",
        "docs/v0-1-0-editor-requirement-source-aliases.json",
        evidence,
    )
}
