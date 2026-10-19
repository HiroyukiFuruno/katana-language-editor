use std::path::Path;

use super::artifact_model::{BranchCatalogArtifact, LeafManifestArtifact, SourceClosureArtifact};
use super::generic_interaction_mapping;
use super::operational_cli::CliOptions;

pub(super) fn enrich_leaf_manifest(options: &CliOptions) -> Result<(), String> {
    let (mut manifest, branches) = load_immutable_inputs(options)?;
    let mapping = Path::new(options.one("mapping")?);
    let output = Path::new(options.one("output")?);
    if output.exists() {
        return Err(format!(
            "refusing to overwrite enriched leaf manifest: {}",
            output.display()
        ));
    }
    let mapping = generic_interaction_mapping::GenericInteractionMapping::read(mapping)?;
    manifest.leafs = manifest
        .canonical_binding
        .enrich_leafs(&branches, &mapping)?;
    write_json(output, &manifest, "enriched leaf manifest")
}

pub(super) fn emit_annotation_catalog(options: &CliOptions) -> Result<(), String> {
    let (manifest, branches) = load_immutable_inputs(options)?;
    let output = Path::new(options.one("output")?);
    if output.exists() {
        return Err(format!(
            "refusing to overwrite generic interaction annotation catalog: {}",
            output.display()
        ));
    }
    let catalog = manifest.canonical_binding.annotation_catalog(&branches)?;
    write_json(output, &catalog, "generic interaction annotation catalog")
}

fn load_immutable_inputs(
    options: &CliOptions,
) -> Result<(LeafManifestArtifact, BranchCatalogArtifact), String> {
    let input = Path::new(options.one("input")?);
    let source_closure = Path::new(options.one("source-closure")?);
    let branches = Path::new(options.one("branches")?);
    let manifest = read_json::<LeafManifestArtifact>(input, "leaf manifest")?;
    let source_closure = read_json::<SourceClosureArtifact>(source_closure, "source closure")?;
    let branches = read_json::<BranchCatalogArtifact>(branches, "branch catalog")?;
    manifest
        .canonical_binding
        .validate(&source_closure, &branches)
        .map_err(|error| format!("leaf manifest canonical binding: {error}"))?;
    Ok((manifest, branches))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path, name: &str) -> Result<T, String> {
    let bytes = std::fs::read(path).map_err(|error| format!("failed to read {name}: {error}"))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("failed to parse {name}: {error}"))
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T, name: &str) -> Result<(), String> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("failed to serialize {name}: {error}"))?;
    bytes.push(b'\n');
    std::fs::write(path, bytes).map_err(|error| format!("failed to write {name}: {error}"))
}
