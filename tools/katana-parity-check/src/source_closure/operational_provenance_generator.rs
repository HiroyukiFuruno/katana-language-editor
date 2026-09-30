use std::fs;
use std::path::Path;

use super::operational_input::EvidenceRef;
use super::operational_output::capture_bytes;

pub(super) fn capture(
    stage_root: &Path,
    generator_schema: &Path,
    run_id: &str,
    evidence: &mut Vec<EvidenceRef>,
) -> Result<(EvidenceRef, EvidenceRef), String> {
    let generator_path = std::env::current_exe()
        .map_err(|error| format!("failed to resolve generator executable: {error}"))?;
    let generator_bytes = fs::read(&generator_path)
        .map_err(|error| format!("failed to read generator executable: {error}"))?;
    let schema_bytes = fs::read(generator_schema)
        .map_err(|error| format!("failed to read generator schema: {error}"))?;
    let binary = capture_bytes(
        stage_root,
        "provenance/generator/generator.bin",
        &generator_bytes,
        run_id,
        "read-only-local-source",
        &generator_path.to_string_lossy(),
        evidence,
    )?;
    let schema = capture_bytes(
        stage_root,
        "provenance/generator/schema.md",
        &schema_bytes,
        run_id,
        "read-only-local-source",
        &generator_schema.to_string_lossy(),
        evidence,
    )?;
    Ok((binary, schema))
}
