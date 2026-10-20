use std::fs;
use std::path::Path;

use super::fingerprint::sha256_hex;
use super::generic_interaction_mapping::GenericInteractionMapping;

mod model;
mod paths;
mod profiles;
mod reconstruction;

use model::{Bundle, SCHEMA};
use paths::safe_file;
use profiles::{index_profiles, validate_profile_fingerprints};

const MAPPING_FILE: &str = "generic-interaction-mapping.json";

pub(super) struct ConsumerReleaseEvidenceVerifier;

impl ConsumerReleaseEvidenceVerifier {
    pub(super) fn validate(
        artifact_root: &Path,
        loaded: &super::load::LoadedArtifacts,
        source_bytes: &[u8],
        bundle_bytes: &[u8],
    ) -> Result<(), String> {
        let bundle: Bundle = serde_json::from_slice(bundle_bytes)
            .map_err(|error| format!("consumer release evidence bundle is invalid: {error}"))?;
        validate_bundle_schema(&bundle)?;
        let source: super::artifact_model::SourceClosureArtifact =
            serde_json::from_slice(source_bytes)
                .map_err(|error| format!("consumer release source closure is invalid: {error}"))?;
        compare_values(
            "source-closure snapshot",
            std::slice::from_ref(&source),
            std::slice::from_ref(&loaded.source_closure),
        )?;

        let root = paths::safe_root(artifact_root)?;
        let source_hash = sha256_hex(source_bytes);
        if bundle.source_closure_sha256 != source_hash {
            return Err("consumer release bundle source closure hash differs".into());
        }
        validate_canonical_binding(loaded)?;

        let mapping_path = safe_file(&root, MAPPING_FILE, "mapping")?;
        let mapping_bytes =
            fs::read(&mapping_path).map_err(|error| format!("mapping cannot be read: {error}"))?;
        if bundle.mapping_sha256 != sha256_hex(&mapping_bytes) {
            return Err("consumer release bundle mapping hash differs".into());
        }
        let mapping: GenericInteractionMapping = serde_json::from_slice(&mapping_bytes)
            .map_err(|error| format!("consumer release mapping is invalid: {error}"))?;
        let enriched = mapping.enrich(
            &loaded.leaf_manifest.canonical_binding,
            &loaded.branch_catalog,
        )?;
        let profiles = index_profiles(&bundle.profiles)?;
        validate_profile_fingerprints(&loaded.source_closure.profiles, &profiles)?;

        let expected = reconstruction::reconstruct(
            &root,
            loaded,
            &mapping,
            &bundle,
            &source_hash,
            &profiles,
            enriched,
        )?;
        compare_values(
            "leaf-manifest leafs",
            &expected.leafs,
            &loaded.leaf_manifest.leafs,
        )?;
        compare_values(
            "execution-record executions",
            &expected.executions,
            &loaded.execution_record.executions,
        )?;
        compare_values(
            "storybook-artifacts",
            &expected.storybook,
            &loaded.storybook.artifacts,
        )
    }
}

fn validate_bundle_schema(bundle: &Bundle) -> Result<(), String> {
    if bundle.schema != SCHEMA {
        return Err(format!(
            "unsupported consumer release evidence schema: {}",
            bundle.schema
        ));
    }
    Ok(())
}

fn validate_canonical_binding(loaded: &super::load::LoadedArtifacts) -> Result<(), String> {
    loaded
        .leaf_manifest
        .canonical_binding
        .validate(&loaded.source_closure, &loaded.branch_catalog)
        .map_err(|error| format!("leaf-manifest canonical binding: {error}"))?;
    let errors = loaded
        .leaf_manifest
        .canonical_binding
        .release_completeness_errors();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "canonical binding is incomplete: {}",
            errors.join("; ")
        ))
    }
}

fn compare_values<T: serde::Serialize>(
    name: &str,
    expected: &[T],
    actual: &[T],
) -> Result<(), String> {
    let expected = expected
        .iter()
        .map(serde_json::to_vec)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("serialize expected {name}: {error}"))?;
    let actual = actual
        .iter()
        .map(serde_json::to_vec)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("serialize actual {name}: {error}"))?;
    if expected == actual {
        Ok(())
    } else {
        Err(format!("{name} differs from consumer reconstruction"))
    }
}

#[cfg(test)]
mod tests;
