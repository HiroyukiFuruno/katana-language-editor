use crate::scenario_manifest_leaf::LeafSpec;
use crate::scenario_manifest_types::EffectClass::{
    InProcessHostEffect as InProcess, NativeExternalHostEffect as NativeExternal,
    NoMutationHostEffect as NoMutation,
};
use crate::scenario_manifest_types::Precondition::{
    QueryCommitted, WorkspaceSearchAvailable as Available, WorkspaceSearchOpen as Open,
};
use crate::scenario_manifest_types::{
    AuditRow, EffectClass, FeatureGroup, FocusLifecycle, FullEditorStepId, InputClass,
    LifecycleRequirement, MutationPolicy, Precondition, SourceMarker,
};

const AVAILABLE_QUERY: &[Precondition] = &[Open, Available, QueryCommitted];

macro_rules! result_step {
    ($name:ident, $prefix:literal, $suffix:literal) => {
        pub(crate) const $name: FullEditorStepId =
            FullEditorStepId::new(concat!("workspace-search.", $prefix, ".", $suffix));
    };
}

result_step!(
    WORKSPACE_SEARCH_FILENAME_RESULT_SELECT,
    "filename",
    "result-select"
);
result_step!(
    WORKSPACE_SEARCH_FILENAME_RESULT_MISSING,
    "filename",
    "result-missing"
);
result_step!(
    WORKSPACE_SEARCH_MARKDOWN_RESULT_JUMP,
    "markdown",
    "result-jump"
);
result_step!(
    WORKSPACE_SEARCH_MARKDOWN_RESULT_MISSING,
    "markdown",
    "result-missing"
);

macro_rules! leaf_spec {
    ($step:ident, $effect:ident, $marker:expr) => {
        result_leaf(
            $step,
            InputClass::HostBoundary,
            $effect,
            AVAILABLE_QUERY,
            $marker,
        )
    };
}

pub(crate) const WORKSPACE_SEARCH_RESULT_LEAF_SPECS: &[LeafSpec] = &[
    leaf_spec!(
        WORKSPACE_SEARCH_FILENAME_RESULT_SELECT,
        InProcess,
        SourceMarker::AuditRow(AuditRow::WorkspaceFilenameResultSelect)
    ),
    leaf_spec!(
        WORKSPACE_SEARCH_FILENAME_RESULT_MISSING,
        NoMutation,
        SourceMarker::AuditRow(AuditRow::WorkspaceFilenameResultSelect)
    ),
    leaf_spec!(
        WORKSPACE_SEARCH_MARKDOWN_RESULT_JUMP,
        NativeExternal,
        SourceMarker::AuditRow(AuditRow::WorkspaceMarkdownResultJump)
    ),
    leaf_spec!(
        WORKSPACE_SEARCH_MARKDOWN_RESULT_MISSING,
        NoMutation,
        SourceMarker::AuditRow(AuditRow::WorkspaceMarkdownResultJump)
    ),
];

const fn result_leaf(
    step_id: FullEditorStepId,
    input_class: InputClass,
    effect_class: EffectClass,
    preconditions: &'static [Precondition],
    source_marker: SourceMarker,
) -> LeafSpec {
    let mutation = match effect_class {
        InProcess => MutationPolicy::HostStateMutationExpected,
        NativeExternal => MutationPolicy::NavigationOnlyMutationExpected,
        _ => MutationPolicy::NoHostMutationRequired,
    };
    LeafSpec {
        step_id,
        feature_group: FeatureGroup::WorkspaceSearchResults,
        input_class,
        effect_class,
        source_marker,
        preconditions,
        lifecycle: LifecycleRequirement::new(FocusLifecycle::FocusRemainsStable, mutation),
        release_blocker: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario_manifest::FullEditorScenarioManifest;
    use crate::scenario_manifest_catalog::required_evidence;
    use crate::scenario_manifest_types::{FeatureGroup, MutationPolicy, SourceMarker};

    const LEAVES: &[FullEditorStepId] = &[
        WORKSPACE_SEARCH_FILENAME_RESULT_SELECT,
        WORKSPACE_SEARCH_FILENAME_RESULT_MISSING,
        WORKSPACE_SEARCH_MARKDOWN_RESULT_JUMP,
        WORKSPACE_SEARCH_MARKDOWN_RESULT_MISSING,
    ];
    const CONFLICTING_FEATURE_GROUPS: &[FeatureGroup] = &[
        FeatureGroup::WorkspaceSearchFilename,
        FeatureGroup::WorkspaceSearchMarkdown,
        FeatureGroup::WorkspaceSearchModal,
        FeatureGroup::DocumentFind,
        FeatureGroup::DocumentFindMarkdownMatching,
        FeatureGroup::DocumentWorkspaceBoundary,
    ];

    #[test]
    fn workspace_search_results_have_expected_ids_and_feature_group() {
        let manifest = FullEditorScenarioManifest::document_find_and_replace_seed();
        let ids = manifest
            .leaves
            .iter()
            .filter(|leaf| leaf.feature_group == FeatureGroup::WorkspaceSearchResults)
            .map(|leaf| leaf.step_id)
            .collect::<Vec<_>>();
        assert_eq!(ids.len(), LEAVES.len());
        assert_eq!(&ids, LEAVES);
    }

    #[test]
    fn workspace_search_results_have_expected_effects_markers_evidence_and_is_disjoint() {
        let manifest = FullEditorScenarioManifest::document_find_and_replace_seed();
        for leaf in manifest
            .leaves
            .iter()
            .filter(|leaf| leaf.feature_group == FeatureGroup::WorkspaceSearchResults)
        {
            let (effect, mutation, marker) = expectation(leaf.step_id);
            assert_eq!(leaf.input_class, InputClass::HostBoundary);
            assert!(!CONFLICTING_FEATURE_GROUPS.contains(&leaf.feature_group));
            assert_eq!(leaf.effect_class, effect);
            assert_eq!(leaf.source_marker, marker);
            assert_eq!(leaf.lifecycle.mutation, mutation);
            assert_eq!(leaf.required_evidence, required_evidence());
            assert!(leaf.release_blocker.is_none());
        }
    }

    fn expectation(step_id: FullEditorStepId) -> (EffectClass, MutationPolicy, SourceMarker) {
        match step_id {
            WORKSPACE_SEARCH_FILENAME_RESULT_SELECT => (
                EffectClass::InProcessHostEffect,
                MutationPolicy::HostStateMutationExpected,
                SourceMarker::AuditRow(AuditRow::WorkspaceFilenameResultSelect),
            ),
            WORKSPACE_SEARCH_FILENAME_RESULT_MISSING => (
                EffectClass::NoMutationHostEffect,
                MutationPolicy::NoHostMutationRequired,
                SourceMarker::AuditRow(AuditRow::WorkspaceFilenameResultSelect),
            ),
            WORKSPACE_SEARCH_MARKDOWN_RESULT_JUMP => (
                EffectClass::NativeExternalHostEffect,
                MutationPolicy::NavigationOnlyMutationExpected,
                SourceMarker::AuditRow(AuditRow::WorkspaceMarkdownResultJump),
            ),
            WORKSPACE_SEARCH_MARKDOWN_RESULT_MISSING => (
                EffectClass::NoMutationHostEffect,
                MutationPolicy::NoHostMutationRequired,
                SourceMarker::AuditRow(AuditRow::WorkspaceMarkdownResultJump),
            ),
            _ => unreachable!(),
        }
    }
}
