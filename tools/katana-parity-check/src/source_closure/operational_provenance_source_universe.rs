use std::{fs, path::Path};

use super::operational_input::EvidenceRef;
use super::operational_output::capture_bytes;

pub(super) fn capture(
    stage_root: &Path,
    source_universe: &Path,
    run_id: &str,
    evidence: &mut Vec<EvidenceRef>,
) -> Result<EvidenceRef, String> {
    let bytes = fs::read(source_universe)
        .map_err(|error| format!("failed to read source-universe input: {error}"))?;
    super::source_roots::validate_source_universe_document(&bytes)?;
    capture_bytes(
        stage_root,
        "provenance/source-universe/katana-editor-source-universe.md",
        &bytes,
        run_id,
        "read-only-local-source",
        "docs/v0-1-0-katana-editor-source-universe.md",
        evidence,
    )
}
