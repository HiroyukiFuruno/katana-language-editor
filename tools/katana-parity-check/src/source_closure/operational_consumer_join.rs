use std::path::Path;

use super::super::artifact_paths::ArtifactPaths;
use super::super::artifact_validator::SourceClosureArtifactValidator;
use super::super::consumer_artifact_intake::ConsumerArtifactIntake;
use super::super::consumer_join_publication::ConsumerJoinPublication;
use super::super::fingerprint::sha256_hex;
use super::super::generic_interaction_mapping::{
    ConsumerPlanProvenance, GenericInteractionMapping,
};
use super::super::operational_cli::CliOptions;
use super::{ImmutableMappingInputs, load_immutable_inputs, write_json};

#[path = "consumer_join_records.rs"]
mod records;

#[cfg(test)]
#[path = "operational_consumer_join_tests.rs"]
mod tests;

const PROFILE_OPTIONS: [(&str, &str); 3] = [
    ("macos-latest", "macos-artifact"),
    ("windows-latest", "windows-artifact"),
    ("ubuntu-latest", "linux-artifact"),
];

pub(super) fn publish(options: &CliOptions) -> Result<(), String> {
    let inputs = load_immutable_inputs(options)?;
    let unresolved = inputs
        .manifest
        .canonical_binding
        .release_completeness_errors();
    if !unresolved.is_empty() {
        return Err(format!(
            "consumer join requires complete source semantics: {}",
            unresolved.join("; ")
        ));
    }
    let mapping_bytes = read_regular(Path::new(options.one("mapping")?))?;
    let mapping: GenericInteractionMapping = serde_json::from_slice(&mapping_bytes)
        .map_err(|error| format!("consumer join mapping is invalid: {error}"))?;
    let run_id = options.one("run-id")?;
    let leafs = mapping.enrich(&inputs.manifest.canonical_binding, &inputs.branches)?;
    let mut records = records::ConsumerJoinRecords::new(leafs);
    ConsumerJoinPublication::publish(Path::new(options.one("output-dir")?), |staging| {
        copy_source_inputs(options, &inputs, staging)?;
        write_bytes(
            &staging.join("generic-interaction-mapping.json"),
            &mapping_bytes,
        )?;
        let context = ProfileCopy {
            inputs: &inputs,
            mapping: &mapping,
            run_id,
            staging,
        };
        let profiles = PROFILE_OPTIONS
            .iter()
            .map(|(profile, key)| context.copy(profile, Path::new(options.one(key)?), &mut records))
            .collect::<Result<Vec<_>, String>>()?;
        records.write(staging, &inputs.manifest)?;
        let bundle = serde_json::json!({
            "schema": "kle.consumer-release-evidence.v1",
            "run_id": run_id,
            "source_closure_sha256": inputs.source_sha256,
            "mapping_sha256": sha256_hex(&mapping_bytes),
            "profiles": profiles,
        });
        write_json(
            &staging.join("kle-release-evidence.json"),
            &bundle,
            "consumer release bundle",
        )?;
        SourceClosureArtifactValidator::validate(&ArtifactPaths::from_dir(staging))
    })
}

struct ProfileCopy<'a> {
    inputs: &'a ImmutableMappingInputs,
    mapping: &'a GenericInteractionMapping,
    run_id: &'a str,
    staging: &'a Path,
}

impl ProfileCopy<'_> {
    fn copy(
        &self,
        profile_id: &str,
        source: &Path,
        records: &mut records::ConsumerJoinRecords,
    ) -> Result<serde_json::Value, String> {
        let profiles = self
            .inputs
            .source
            .profiles
            .iter()
            .filter(|profile| profile.id == profile_id)
            .collect::<Vec<_>>();
        let [profile] = profiles.as_slice() else {
            return Err("consumer join requires one source profile record".into());
        };
        let plan = self.mapping.consumer_plan(
            &self.inputs.manifest.canonical_binding,
            &self.inputs.branches,
            profile_id,
            ConsumerPlanProvenance {
                run_id: self.run_id.to_owned(),
                source_closure_sha256: self.inputs.source_sha256.clone(),
                profile_fingerprint: profile.fingerprint.clone(),
            },
        )?;
        let plan_bytes = serde_json::to_vec(&plan).map_err(|error| error.to_string())?;
        let validated = ConsumerArtifactIntake::validate_profile_artifact(&plan_bytes, source)?;
        let destination = self.staging.join("consumer").join(profile_id);
        std::fs::create_dir_all(&destination).map_err(|error| error.to_string())?;
        for stage in validated.stages() {
            for suffix in [".png", ".consumer-artifact.json", ".unicode-evidence.json"] {
                let filename = format!("{}{suffix}", stage.stage_id);
                write_bytes(
                    &destination.join(&filename),
                    &read_regular(&source.join(filename))?,
                )?;
            }
        }
        records.add_profile(profile_id, &validated)?;
        let evidence_bytes = read_regular(&source.join("kle-consumer-artifact-evidence.json"))?;
        write_bytes(
            &destination.join("kle-consumer-artifact-evidence.json"),
            &evidence_bytes,
        )?;
        ConsumerArtifactIntake::validate_profile_artifact(&plan_bytes, &destination)?;
        Ok(serde_json::json!({
            "profile_id": profile_id, "profile_fingerprint": profile.fingerprint,
            "plan_sha256": sha256_hex(&plan_bytes), "evidence_sha256": sha256_hex(&evidence_bytes),
        }))
    }
}

fn copy_source_inputs(
    options: &CliOptions,
    inputs: &ImmutableMappingInputs,
    staging: &Path,
) -> Result<(), String> {
    let source = read_regular(Path::new(options.one("source-closure")?))?;
    if sha256_hex(&source) != inputs.source_sha256 {
        return Err("source closure changed while preparing consumer join".into());
    }
    write_bytes(&staging.join("source-closure.json"), &source)?;
    write_bytes(
        &staging.join("branch-catalog.json"),
        &read_regular(Path::new(options.one("branches")?))?,
    )?;
    let parent = Path::new(options.one("input")?)
        .parent()
        .ok_or("leaf manifest has no parent")?;
    for name in [
        "action-origins.json",
        "context-menu-target-manifest.json",
        "source-derived-native-target.json",
    ] {
        let path = parent.join(name);
        match std::fs::symlink_metadata(&path) {
            Err(error)
                if error.kind() == std::io::ErrorKind::NotFound
                    && name != "action-origins.json" => {}
            _ => write_bytes(&staging.join(name), &read_regular(&path)?)?,
        }
    }
    Ok(())
}

fn read_regular(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(format!(
            "consumer join requires a regular source file: {}",
            path.display()
        ));
    }
    std::fs::read(path).map_err(|error| error.to_string())
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| error.to_string())?;
    file.write_all(bytes).map_err(|error| error.to_string())
}
