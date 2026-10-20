use crate::scenario_manifest_leaf::LeafSpec;
use crate::scenario_manifest_types::EffectClass::{
    InProcessHostEffect as InProcess, NoMutationHostEffect as NoMutation,
};
use crate::scenario_manifest_types::InputClass::{HostBoundary, Keyboard, QueryState, TextEdit};
use crate::scenario_manifest_types::Precondition::{
    QueryCommitted, QueryHasNoMatches, WorkspaceSearchAvailable as Available,
    WorkspaceSearchOpen as Open, WorkspaceSearchUnavailable as Unavailable,
};
use crate::scenario_manifest_types::{
    AuditRow, EffectClass, FeatureGroup, FocusLifecycle, FullEditorStepId, InputClass,
    LifecycleRequirement, MutationPolicy, Precondition, SourceMarker,
};

const AVAILABLE: &[Precondition] = &[Open, Available];
const COMMITTED: &[Precondition] = &[Open, Available, QueryCommitted];
const NO_MATCHES: &[Precondition] = &[Open, Available, QueryCommitted, QueryHasNoMatches];
const UNAVAILABLE: &[Precondition] = &[Open, Unavailable];

macro_rules! markdown_step {
    ($name:ident, $suffix:literal) => {
        pub(crate) const $name: FullEditorStepId =
            FullEditorStepId::new(concat!("workspace-search.markdown.", $suffix));
    };
}

markdown_step!(WORKSPACE_SEARCH_MARKDOWN_QUERY_EDIT, "query-edit");
markdown_step!(WORKSPACE_SEARCH_MARKDOWN_ENTER_REFRESH, "enter-refresh");
markdown_step!(
    WORKSPACE_SEARCH_MARKDOWN_LOST_FOCUS_REFRESH,
    "lost-focus-refresh"
);
markdown_step!(WORKSPACE_SEARCH_MARKDOWN_HISTORY_SELECT, "history-select");
markdown_step!(WORKSPACE_SEARCH_MARKDOWN_HISTORY_REMOVE, "history-remove");
markdown_step!(WORKSPACE_SEARCH_MARKDOWN_HISTORY_CLEAR, "history-clear");
markdown_step!(WORKSPACE_SEARCH_MARKDOWN_EMPTY_HISTORY, "empty-history");
markdown_step!(WORKSPACE_SEARCH_MARKDOWN_NO_WORKSPACE, "no-workspace");
markdown_step!(WORKSPACE_SEARCH_MARKDOWN_NO_RESULT, "no-result");

const QUERY_EDIT: FullEditorStepId = WORKSPACE_SEARCH_MARKDOWN_QUERY_EDIT;
const ENTER_REFRESH: FullEditorStepId = WORKSPACE_SEARCH_MARKDOWN_ENTER_REFRESH;
const LOST_FOCUS_REFRESH: FullEditorStepId = WORKSPACE_SEARCH_MARKDOWN_LOST_FOCUS_REFRESH;
const HISTORY_SELECT: FullEditorStepId = WORKSPACE_SEARCH_MARKDOWN_HISTORY_SELECT;
const HISTORY_REMOVE: FullEditorStepId = WORKSPACE_SEARCH_MARKDOWN_HISTORY_REMOVE;
const HISTORY_CLEAR: FullEditorStepId = WORKSPACE_SEARCH_MARKDOWN_HISTORY_CLEAR;
const EMPTY_HISTORY: FullEditorStepId = WORKSPACE_SEARCH_MARKDOWN_EMPTY_HISTORY;
const NO_WORKSPACE: FullEditorStepId = WORKSPACE_SEARCH_MARKDOWN_NO_WORKSPACE;
const NO_RESULT: FullEditorStepId = WORKSPACE_SEARCH_MARKDOWN_NO_RESULT;

macro_rules! leaf_spec {
    ($step:ident, $input:ident, $effect:ident, $pre:ident) => {
        markdown_leaf($step, $input, $effect, $pre)
    };
}

pub(crate) const WORKSPACE_SEARCH_MARKDOWN_LEAF_SPECS: &[LeafSpec] = &[
    leaf_spec!(QUERY_EDIT, TextEdit, InProcess, AVAILABLE),
    leaf_spec!(ENTER_REFRESH, Keyboard, InProcess, COMMITTED),
    leaf_spec!(LOST_FOCUS_REFRESH, HostBoundary, InProcess, COMMITTED),
    leaf_spec!(HISTORY_SELECT, QueryState, InProcess, AVAILABLE),
    leaf_spec!(HISTORY_REMOVE, QueryState, InProcess, AVAILABLE),
    leaf_spec!(HISTORY_CLEAR, QueryState, InProcess, AVAILABLE),
    leaf_spec!(EMPTY_HISTORY, QueryState, NoMutation, AVAILABLE),
    leaf_spec!(NO_WORKSPACE, HostBoundary, NoMutation, UNAVAILABLE),
    leaf_spec!(NO_RESULT, HostBoundary, NoMutation, NO_MATCHES),
];

const fn markdown_leaf(
    step_id: FullEditorStepId,
    input_class: InputClass,
    effect_class: EffectClass,
    preconditions: &'static [Precondition],
) -> LeafSpec {
    let mutation = match effect_class {
        InProcess => MutationPolicy::HostStateMutationExpected,
        _ => MutationPolicy::NoHostMutationRequired,
    };
    LeafSpec {
        step_id,
        feature_group: FeatureGroup::WorkspaceSearchMarkdown,
        input_class,
        effect_class,
        source_marker: SourceMarker::AuditRow(AuditRow::WorkspaceMarkdownHistoryFamily),
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
    use crate::scenario_manifest_types::FullEditorScenarioLeaf;
    use std::collections::HashSet;

    #[test]
    fn workspace_markdown_content_search_has_exactly_nine_host_only_leaves() {
        let manifest = FullEditorScenarioManifest::document_find_and_replace_seed();
        let markdown = leaf_ids(&manifest, &[FeatureGroup::WorkspaceSearchMarkdown]);
        assert_eq!(markdown.len(), [(); 9].len());
        assert!(
            WORKSPACE_SEARCH_MARKDOWN_LEAF_SPECS
                .iter()
                .all(|spec| markdown.contains(&spec.step_id))
        );
        assert!(
            markdown
                .iter()
                .all(|id| id.as_str().starts_with("workspace-search.markdown."))
        );
    }

    #[test]
    fn workspace_markdown_content_search_uses_generic_group_marker_input_and_evidence() {
        let manifest = FullEditorScenarioManifest::document_find_and_replace_seed();
        for leaf in markdown_leaves(&manifest) {
            assert_eq!(leaf.feature_group, FeatureGroup::WorkspaceSearchMarkdown);
            assert_eq!(
                leaf.source_marker,
                SourceMarker::AuditRow(AuditRow::WorkspaceMarkdownHistoryFamily)
            );
            assert!([TextEdit, Keyboard, HostBoundary, QueryState].contains(&leaf.input_class));
            assert_eq!(leaf.required_evidence, required_evidence());
            assert!(leaf.release_blocker.is_none());
        }
    }

    #[test]
    fn workspace_markdown_content_search_no_mutation_cases_are_explicit() {
        let manifest = FullEditorScenarioManifest::document_find_and_replace_seed();
        let no_mutation = [
            WORKSPACE_SEARCH_MARKDOWN_EMPTY_HISTORY,
            WORKSPACE_SEARCH_MARKDOWN_NO_WORKSPACE,
            WORKSPACE_SEARCH_MARKDOWN_NO_RESULT,
        ];
        for leaf in markdown_leaves(&manifest) {
            let expected = if no_mutation.contains(&leaf.step_id) {
                (NoMutation, MutationPolicy::NoHostMutationRequired)
            } else {
                (InProcess, MutationPolicy::HostStateMutationExpected)
            };
            assert_eq!((leaf.effect_class, leaf.lifecycle.mutation), expected);
        }
    }

    #[test]
    fn workspace_markdown_content_search_is_disjoint_from_document_find_and_filename() {
        let manifest = FullEditorScenarioManifest::document_find_and_replace_seed();
        let markdown = leaf_ids(&manifest, &[FeatureGroup::WorkspaceSearchMarkdown]);
        let document = leaf_ids(
            &manifest,
            &[
                FeatureGroup::DocumentFind,
                FeatureGroup::DocumentFindMarkdownMatching,
                FeatureGroup::DocumentWorkspaceBoundary,
            ],
        );
        let filename = leaf_ids(&manifest, &[FeatureGroup::WorkspaceSearchFilename]);
        assert!(markdown.is_disjoint(&document));
        assert!(markdown.is_disjoint(&filename));
    }

    fn markdown_leaves(manifest: &FullEditorScenarioManifest) -> Vec<&FullEditorScenarioLeaf> {
        manifest
            .leaves
            .iter()
            .filter(|leaf| leaf.feature_group == FeatureGroup::WorkspaceSearchMarkdown)
            .collect()
    }

    fn leaf_ids(
        manifest: &FullEditorScenarioManifest,
        groups: &[FeatureGroup],
    ) -> HashSet<FullEditorStepId> {
        manifest
            .leaves
            .iter()
            .filter(|leaf| groups.contains(&leaf.feature_group))
            .map(|leaf| leaf.step_id)
            .collect()
    }
}
