use std::io::Write;
use std::path::Path;

use super::artifact_model::{BranchCatalogArtifact, LeafManifestArtifact, SourceClosureArtifact};
use super::generic_interaction_mapping;
use super::operational_cli::CliOptions;

struct ImmutableMappingInputs {
    manifest: LeafManifestArtifact,
    branches: BranchCatalogArtifact,
    source: SourceClosureArtifact,
    source_sha256: String,
}

pub(super) fn emit_consumer_plan(options: &CliOptions) -> Result<(), String> {
    let output = Path::new(options.one("output")?);
    if output.exists() {
        return Err(format!(
            "refusing to overwrite consumer plan: {}",
            output.display()
        ));
    }
    let plan = verified_consumer_plan(options)?;
    write_json(output, &plan, "consumer plan")
}

pub(super) fn validate_consumer_artifact(options: &CliOptions) -> Result<(), String> {
    let plan = verified_consumer_plan(options)?;
    let bytes = serde_json::to_vec(&plan)
        .map_err(|error| format!("failed to serialize verified consumer plan: {error}"))?;
    super::consumer_artifact_intake::ConsumerArtifactIntake::validate_profile_artifact(
        &bytes,
        Path::new(options.one("artifact-dir")?),
    )
}

fn verified_consumer_plan(
    options: &CliOptions,
) -> Result<generic_interaction_mapping::ConsumerArtifactPlan, String> {
    let inputs = load_immutable_inputs(options)?;
    let mapping = generic_interaction_mapping::GenericInteractionMapping::read(Path::new(
        options.one("mapping")?,
    ))?;
    let profile = options.one("profile-id")?;
    let profiles = inputs
        .source
        .profiles
        .iter()
        .filter(|record| record.id == profile)
        .collect::<Vec<_>>();
    let [record] = profiles.as_slice() else {
        return Err("consumer plan requires exactly one matching source profile".into());
    };
    let provenance = generic_interaction_mapping::ConsumerPlanProvenance {
        run_id: options.one("run-id")?.to_owned(),
        source_closure_sha256: inputs.source_sha256,
        profile_fingerprint: record.fingerprint.clone(),
    };
    mapping.consumer_plan(
        &inputs.manifest.canonical_binding,
        &inputs.branches,
        profile,
        provenance,
    )
}

pub(super) fn enrich_leaf_manifest(options: &CliOptions) -> Result<(), String> {
    let mut inputs = load_immutable_inputs(options)?;
    let mapping = Path::new(options.one("mapping")?);
    let output = Path::new(options.one("output")?);
    if output.exists() {
        return Err(format!(
            "refusing to overwrite enriched leaf manifest: {}",
            output.display()
        ));
    }
    let mapping = generic_interaction_mapping::GenericInteractionMapping::read(mapping)?;
    inputs.manifest.leafs = inputs
        .manifest
        .canonical_binding
        .enrich_leafs(&inputs.branches, &mapping)?;
    write_json(output, &inputs.manifest, "enriched leaf manifest")
}

pub(super) fn emit_annotation_catalog(options: &CliOptions) -> Result<(), String> {
    let inputs = load_immutable_inputs(options)?;
    let output = Path::new(options.one("output")?);
    if output.exists() {
        return Err(format!(
            "refusing to overwrite generic interaction annotation catalog: {}",
            output.display()
        ));
    }
    let catalog = inputs
        .manifest
        .canonical_binding
        .annotation_catalog(&inputs.branches)?;
    write_json(output, &catalog, "generic interaction annotation catalog")
}

fn load_immutable_inputs(options: &CliOptions) -> Result<ImmutableMappingInputs, String> {
    let input = Path::new(options.one("input")?);
    let source_closure = Path::new(options.one("source-closure")?);
    let branches = Path::new(options.one("branches")?);
    let manifest = read_json::<LeafManifestArtifact>(input, "leaf manifest")?;
    let source_bytes = std::fs::read(source_closure)
        .map_err(|error| format!("failed to read source closure: {error}"))?;
    let source_closure: SourceClosureArtifact = serde_json::from_slice(&source_bytes)
        .map_err(|error| format!("failed to parse source closure: {error}"))?;
    let branches = read_json::<BranchCatalogArtifact>(branches, "branch catalog")?;
    super::requirement_binding::RequirementBindingResult::ensure_same_root(
        &source_closure.root,
        &manifest.root,
    )?;
    manifest
        .canonical_binding
        .validate(&source_closure, &branches)
        .map_err(|error| format!("leaf manifest canonical binding: {error}"))?;
    Ok(ImmutableMappingInputs {
        manifest,
        branches,
        source: source_closure,
        source_sha256: super::fingerprint::sha256_hex(&source_bytes),
    })
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path, name: &str) -> Result<T, String> {
    let bytes = std::fs::read(path).map_err(|error| format!("failed to read {name}: {error}"))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("failed to parse {name}: {error}"))
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T, name: &str) -> Result<(), String> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("failed to serialize {name}: {error}"))?;
    bytes.push(b'\n');
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("failed to create {name}: {error}"))?;
    file.write_all(&bytes)
        .map_err(|error| format!("failed to write {name}: {error}"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn immutable_output_does_not_replace_an_existing_file() -> Result<(), String> {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "kle-mapping-output-{}-{nonce}.json",
            std::process::id()
        ));
        super::write_json(&path, &vec!["original"], "test output")?;
        let replacement = super::write_json(&path, &vec!["replacement"], "test output");
        let bytes = std::fs::read(&path).map_err(|error| error.to_string())?;
        std::fs::remove_file(&path).map_err(|error| error.to_string())?;
        assert!(replacement.is_err());
        let actual: Vec<String> =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        assert_eq!(actual, ["original"]);
        Ok(())
    }
}
