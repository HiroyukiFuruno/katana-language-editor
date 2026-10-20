use super::model::{CANONICAL_PROFILE_IDS, SourceClosureArtifact};
use super::validator_helpers::{
    duplicates, is_canonical_profile_set, is_placeholder, kle_mounted_in_katana,
};

#[path = "source_validation/external.rs"]
mod external;

pub(super) fn validate(source: &SourceClosureArtifact, errors: &mut Vec<String>) {
    external::validate(&source.external_ui_semantic_dependencies, errors);
    if !source.unresolved_edges.is_empty() {
        errors.push(format!(
            "source-closure contains unresolved edges: {}",
            source.unresolved_edges.len()
        ));
    }
    if source.profiles.len() != CANONICAL_PROFILE_IDS.len() {
        errors.push(format!(
            "source-closure must contain {} profiles, got {}",
            CANONICAL_PROFILE_IDS.len(),
            source.profiles.len()
        ));
    }
    if duplicates(source.profiles.iter().map(|profile| &profile.id)) {
        errors.push("source-closure contains duplicate profile ids".to_string());
    }
    if source.profiles.iter().any(|profile| {
        !is_canonical_profile_set(&profile.id) || is_placeholder(profile.id.as_str())
    }) {
        errors.push("source-closure has invalid profile ids".to_string());
    }
    if source.profiles.iter().any(|profile| {
        profile.rustc_host_triple.trim().is_empty()
            || profile.rustc_cfg_sha256.trim().is_empty()
            || profile.fingerprint.trim().is_empty()
            || is_placeholder(profile.fingerprint.as_str())
    }) {
        errors.push("source-closure has placeholder profile values".to_string());
    }
    if source
        .files
        .iter()
        .any(|entry| is_placeholder(entry.path.as_str()) || is_placeholder(entry.sha256.as_str()))
    {
        errors.push("source-closure has placeholder file entries".to_string());
    }
    if duplicates(source.files.iter().map(|entry| &entry.path)) {
        errors.push("source-closure has duplicate file paths".to_string());
    }
    if source
        .files
        .iter()
        .any(|entry| kle_mounted_in_katana(entry.path.as_str()))
    {
        errors.push("source-closure asserts KLE mounted inside KatanA sources".to_string());
    }
}
