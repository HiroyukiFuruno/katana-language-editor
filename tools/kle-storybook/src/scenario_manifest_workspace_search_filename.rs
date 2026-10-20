use crate::scenario_manifest_types::{FullEditorStepId, Precondition};

pub(crate) const AVAILABLE_QUERY: &[Precondition] = &[
    Precondition::WorkspaceSearchOpen,
    Precondition::WorkspaceSearchAvailable,
    Precondition::QueryCommitted,
];
pub(crate) const EMPTY_QUERY: &[Precondition] =
    &[Precondition::WorkspaceSearchOpen, Precondition::QueryEmpty];
pub(crate) const UNAVAILABLE: &[Precondition] = &[
    Precondition::WorkspaceSearchOpen,
    Precondition::WorkspaceSearchUnavailable,
];

pub(crate) const WORKSPACE_SEARCH_FILENAME_QUERY: FullEditorStepId =
    FullEditorStepId::new("workspace-search.filename.query");
pub(crate) const WORKSPACE_SEARCH_FILENAME_INCLUDE_PATTERN: FullEditorStepId =
    FullEditorStepId::new("workspace-search.filename.include-pattern");
pub(crate) const WORKSPACE_SEARCH_FILENAME_EXCLUDE_PATTERN: FullEditorStepId =
    FullEditorStepId::new("workspace-search.filename.exclude-pattern");
pub(crate) const WORKSPACE_SEARCH_FILENAME_MATCH_CASE: FullEditorStepId =
    FullEditorStepId::new("workspace-search.filename.match-case");
pub(crate) const WORKSPACE_SEARCH_FILENAME_MATCH_WORD: FullEditorStepId =
    FullEditorStepId::new("workspace-search.filename.match-word");
pub(crate) const WORKSPACE_SEARCH_FILENAME_REGEX_VALID: FullEditorStepId =
    FullEditorStepId::new("workspace-search.filename.regex-valid");
pub(crate) const WORKSPACE_SEARCH_FILENAME_EMPTY_QUERY: FullEditorStepId =
    FullEditorStepId::new("workspace-search.filename.empty-query");
pub(crate) const WORKSPACE_SEARCH_FILENAME_INVALID_REGEX: FullEditorStepId =
    FullEditorStepId::new("workspace-search.filename.invalid-regex");
pub(crate) const WORKSPACE_SEARCH_FILENAME_NO_WORKSPACE: FullEditorStepId =
    FullEditorStepId::new("workspace-search.filename.no-workspace");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario_manifest::FullEditorScenarioManifest;
    use crate::scenario_manifest_catalog::required_evidence;
    use crate::scenario_manifest_types::{
        AuditRow, EffectClass, FeatureGroup, MutationPolicy, SourceMarker,
    };
    use std::collections::HashSet;

    const FILENAME: &[FullEditorStepId] = &[
        WORKSPACE_SEARCH_FILENAME_QUERY,
        WORKSPACE_SEARCH_FILENAME_INCLUDE_PATTERN,
        WORKSPACE_SEARCH_FILENAME_EXCLUDE_PATTERN,
        WORKSPACE_SEARCH_FILENAME_MATCH_CASE,
        WORKSPACE_SEARCH_FILENAME_MATCH_WORD,
        WORKSPACE_SEARCH_FILENAME_REGEX_VALID,
        WORKSPACE_SEARCH_FILENAME_EMPTY_QUERY,
        WORKSPACE_SEARCH_FILENAME_INVALID_REGEX,
        WORKSPACE_SEARCH_FILENAME_NO_WORKSPACE,
    ];

    fn filename_leaves(
        manifest: &FullEditorScenarioManifest,
    ) -> Vec<&crate::scenario_manifest_types::FullEditorScenarioLeaf> {
        manifest
            .leaves
            .iter()
            .filter(|leaf| FILENAME.contains(&leaf.step_id))
            .collect()
    }

    fn feature_ids(
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

    #[test]
    fn filename_has_nine_unique_leaves_without_other_search_overlap() {
        let manifest = FullEditorScenarioManifest::document_find_and_replace_seed();
        let actual = filename_leaves(&manifest);
        let ids = actual
            .iter()
            .map(|leaf| leaf.step_id)
            .collect::<HashSet<_>>();
        let document = feature_ids(
            &manifest,
            &[
                FeatureGroup::DocumentFind,
                FeatureGroup::DocumentFindMarkdownMatching,
                FeatureGroup::DocumentWorkspaceBoundary,
            ],
        );
        let modal = feature_ids(&manifest, &[FeatureGroup::WorkspaceSearchModal]);
        assert_eq!(actual.len(), 9);
        assert_eq!(ids.len(), 9);
        assert!(
            FILENAME
                .iter()
                .all(|id| ids.contains(id) && !document.contains(id) && !modal.contains(id))
        );
    }

    #[test]
    fn filename_leaves_have_four_evidence_and_shared_source_marker() {
        let manifest = FullEditorScenarioManifest::document_find_and_replace_seed();
        let actual = filename_leaves(&manifest);
        assert!(
            actual
                .iter()
                .all(|leaf| leaf.required_evidence == required_evidence()
                    && matches!(
                        leaf.source_marker,
                        SourceMarker::AuditRow(AuditRow::WorkspaceFilenameFilterFamily)
                    ))
        );
    }

    #[test]
    fn invalid_empty_and_no_workspace_are_no_mutation() {
        let manifest = FullEditorScenarioManifest::document_find_and_replace_seed();
        for id in [
            WORKSPACE_SEARCH_FILENAME_EMPTY_QUERY,
            WORKSPACE_SEARCH_FILENAME_INVALID_REGEX,
            WORKSPACE_SEARCH_FILENAME_NO_WORKSPACE,
        ] {
            let leaf = manifest.leaves.iter().find(|leaf| leaf.step_id == id);
            assert!(leaf.is_some_and(|leaf| matches!(
                leaf.effect_class,
                EffectClass::NoMutationHostEffect
            ) && matches!(
                leaf.lifecycle.mutation,
                MutationPolicy::NoHostMutationRequired
            )));
        }
    }
}
