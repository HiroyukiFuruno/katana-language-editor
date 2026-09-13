use crate::scenario_manifest::{
    FIXED_KATANA_SOURCE_REVISION, FullEditorScenarioManifest, SourceRepository,
};
use crate::scenario_manifest_catalog::required_evidence;
use crate::scenario_manifest_document_find::{
    FIND_CLOSE_ACCESS_KIT, FIND_CLOSE_ESCAPE, FIND_CLOSE_POINTER, FIND_FOCUS_RESTORE,
    FIND_HTML_EXCLUSION, FIND_INLINE_CODE_HIT, FIND_MARKDOWN_TEXT_HIT, FIND_NEXT_ACCESS_KIT,
    FIND_NEXT_KEYBOARD, FIND_NEXT_POINTER, FIND_NEXT_WRAP, FIND_OPEN_KEYBOARD,
    FIND_PREV_ACCESS_KIT, FIND_PREV_KEYBOARD, FIND_PREV_POINTER, FIND_PREV_WRAP, FIND_QUERY_EMPTY,
    FIND_QUERY_IME_COMMIT, FIND_QUERY_NO_RESULT, FIND_QUERY_TEXT, FIND_URL_EXCLUSION,
    FIND_WORKSPACE_BOUNDARY, FIND_ZERO_MATCH_NAVIGATION,
};
use crate::scenario_manifest_replace::{
    REPLACE_ALL_RELEASE_BLOCKER, REPLACE_RELEASE_BLOCKER_GENERAL,
};
use crate::scenario_manifest_types::{FullEditorScenarioLeaf, FullEditorStepId};
use std::collections::HashSet;

const DOCUMENT_FIND_STEPS: &[FullEditorStepId] = &[
    FIND_OPEN_KEYBOARD,
    FIND_CLOSE_ESCAPE,
    FIND_CLOSE_POINTER,
    FIND_CLOSE_ACCESS_KIT,
    FIND_QUERY_TEXT,
    FIND_QUERY_IME_COMMIT,
    FIND_QUERY_EMPTY,
    FIND_QUERY_NO_RESULT,
    FIND_NEXT_POINTER,
    FIND_NEXT_KEYBOARD,
    FIND_NEXT_ACCESS_KIT,
    FIND_PREV_POINTER,
    FIND_PREV_KEYBOARD,
    FIND_PREV_ACCESS_KIT,
    FIND_NEXT_WRAP,
    FIND_PREV_WRAP,
    FIND_ZERO_MATCH_NAVIGATION,
    FIND_FOCUS_RESTORE,
    FIND_MARKDOWN_TEXT_HIT,
    FIND_INLINE_CODE_HIT,
    FIND_URL_EXCLUSION,
    FIND_HTML_EXCLUSION,
    FIND_WORKSPACE_BOUNDARY,
];
const REPLACE_STEPS: &[FullEditorStepId] =
    &[REPLACE_RELEASE_BLOCKER_GENERAL, REPLACE_ALL_RELEASE_BLOCKER];

#[test]
fn seed_uses_fixed_revision_identity() {
    let manifest = FullEditorScenarioManifest::document_find_and_replace_seed();
    assert_eq!(
        manifest.source_revision.revision,
        FIXED_KATANA_SOURCE_REVISION
    );
    assert_eq!(
        manifest.source_revision.repository,
        SourceRepository::FixedKatanaSource
    );
}

#[test]
fn step_ids_are_unique() {
    let manifest = FullEditorScenarioManifest::document_find_and_replace_seed();
    assert!(
        manifest
            .leaves
            .iter()
            .all(|leaf| !leaf.step_id.as_str().is_empty())
    );
    let unique = manifest
        .leaves
        .iter()
        .map(|leaf| leaf.step_id)
        .collect::<HashSet<_>>();
    assert_eq!(unique.len(), manifest.leaves.len());
}

#[test]
fn catalog_step_ids_are_non_empty_and_unique() {
    let mut unique = HashSet::new();
    for step_id in DOCUMENT_FIND_STEPS.iter().chain(REPLACE_STEPS) {
        assert!(!step_id.as_str().is_empty());
        assert!(unique.insert(*step_id));
    }
}

#[test]
fn every_leaf_requires_the_four_evidence_layers() {
    let manifest = FullEditorScenarioManifest::document_find_and_replace_seed();
    assert!(
        manifest
            .leaves
            .iter()
            .all(|leaf| leaf.required_evidence == required_evidence())
    );
}

#[test]
fn document_find_expected_leaf_categories_are_present() {
    let manifest = FullEditorScenarioManifest::document_find_and_replace_seed();
    let actual = manifest
        .leaves
        .iter()
        .map(|leaf| leaf.step_id)
        .collect::<HashSet<_>>();
    assert!(
        DOCUMENT_FIND_STEPS
            .iter()
            .all(|step_id| actual.contains(step_id))
    );
}

#[test]
fn replace_leaves_are_release_blockers_and_cannot_succeed() {
    let manifest = FullEditorScenarioManifest::document_find_and_replace_seed();
    let blockers = manifest
        .leaves
        .iter()
        .filter_map(|leaf| leaf.release_blocker);
    assert_eq!(blockers.clone().count(), [(), ()].len());
    assert!(blockers.clone().all(|blocker| !blocker.success_possible));
    assert!(
        blockers
            .clone()
            .all(|blocker| blocker.host_specification_required)
    );
    assert!(
        blockers
            .clone()
            .all(|blocker| !blocker.local_semantics_required)
    );
}

#[test]
fn manifest_api_exposes_only_generic_categorical_structure() {
    let manifest = FullEditorScenarioManifest::document_find_and_replace_seed();
    assert!(!manifest.leaves.is_empty());
    for leaf in manifest.leaves.iter().take([()].len()) {
        let FullEditorScenarioLeaf {
            step_id,
            feature_group,
            input_class,
            effect_class,
            source_marker,
            preconditions,
            lifecycle,
            required_evidence,
            release_blocker,
        } = leaf;
        let _ = (
            step_id,
            feature_group,
            input_class,
            effect_class,
            source_marker,
            preconditions,
            lifecycle,
            required_evidence,
            release_blocker,
        );
    }
}
