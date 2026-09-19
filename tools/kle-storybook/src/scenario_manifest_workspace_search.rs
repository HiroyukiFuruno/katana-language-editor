use crate::scenario_manifest_leaf::LeafSpec;
use crate::scenario_manifest_types::{
    AuditRow, EffectClass, FeatureGroup, FocusLifecycle, FullEditorStepId, InputClass,
    LifecycleRequirement, MutationPolicy, Precondition, SourceMarker,
};

#[path = "scenario_manifest_workspace_search_filename.rs"]
mod filename;
pub(crate) use filename::{
    WORKSPACE_SEARCH_FILENAME_EMPTY_QUERY, WORKSPACE_SEARCH_FILENAME_EXCLUDE_PATTERN,
    WORKSPACE_SEARCH_FILENAME_INCLUDE_PATTERN, WORKSPACE_SEARCH_FILENAME_INVALID_REGEX,
    WORKSPACE_SEARCH_FILENAME_MATCH_CASE, WORKSPACE_SEARCH_FILENAME_MATCH_WORD,
    WORKSPACE_SEARCH_FILENAME_NO_WORKSPACE, WORKSPACE_SEARCH_FILENAME_QUERY,
    WORKSPACE_SEARCH_FILENAME_REGEX_VALID,
};

const OPEN: &[Precondition] = &[Precondition::WorkspaceSearchOpen];

pub(crate) const WORKSPACE_SEARCH_MODAL_TABS_POINTER: FullEditorStepId =
    FullEditorStepId::new("workspace-search.modal-tabs.pointer");
pub(crate) const WORKSPACE_SEARCH_MODAL_TABS_KEYBOARD: FullEditorStepId =
    FullEditorStepId::new("workspace-search.modal-tabs.keyboard");
pub(crate) const WORKSPACE_SEARCH_MODAL_TABS_ACCESS_KIT: FullEditorStepId =
    FullEditorStepId::new("workspace-search.modal-tabs.accesskit");

pub(crate) const WORKSPACE_SEARCH_MODAL_TAB_LEAF_SPECS: &[LeafSpec] = &[
    modal_tab_leaf(WORKSPACE_SEARCH_MODAL_TABS_POINTER, InputClass::Pointer),
    modal_tab_leaf(WORKSPACE_SEARCH_MODAL_TABS_KEYBOARD, InputClass::Keyboard),
    modal_tab_leaf(
        WORKSPACE_SEARCH_MODAL_TABS_ACCESS_KIT,
        InputClass::AccessKit,
    ),
    filename_leaf(
        WORKSPACE_SEARCH_FILENAME_QUERY,
        InputClass::TextEdit,
        EffectClass::InProcessHostEffect,
        filename::AVAILABLE_QUERY,
    ),
    filename_leaf(
        WORKSPACE_SEARCH_FILENAME_INCLUDE_PATTERN,
        InputClass::TextEdit,
        EffectClass::InProcessHostEffect,
        filename::AVAILABLE_QUERY,
    ),
    filename_leaf(
        WORKSPACE_SEARCH_FILENAME_EXCLUDE_PATTERN,
        InputClass::TextEdit,
        EffectClass::InProcessHostEffect,
        filename::AVAILABLE_QUERY,
    ),
    filename_leaf(
        WORKSPACE_SEARCH_FILENAME_MATCH_CASE,
        InputClass::QueryState,
        EffectClass::InProcessHostEffect,
        filename::AVAILABLE_QUERY,
    ),
    filename_leaf(
        WORKSPACE_SEARCH_FILENAME_MATCH_WORD,
        InputClass::QueryState,
        EffectClass::InProcessHostEffect,
        filename::AVAILABLE_QUERY,
    ),
    filename_leaf(
        WORKSPACE_SEARCH_FILENAME_REGEX_VALID,
        InputClass::QueryState,
        EffectClass::InProcessHostEffect,
        filename::AVAILABLE_QUERY,
    ),
    filename_leaf(
        WORKSPACE_SEARCH_FILENAME_EMPTY_QUERY,
        InputClass::QueryState,
        EffectClass::NoMutationHostEffect,
        filename::EMPTY_QUERY,
    ),
    filename_leaf(
        WORKSPACE_SEARCH_FILENAME_INVALID_REGEX,
        InputClass::QueryState,
        EffectClass::NoMutationHostEffect,
        filename::AVAILABLE_QUERY,
    ),
    filename_leaf(
        WORKSPACE_SEARCH_FILENAME_NO_WORKSPACE,
        InputClass::QueryState,
        EffectClass::NoMutationHostEffect,
        filename::UNAVAILABLE,
    ),
];

const fn modal_tab_leaf(step_id: FullEditorStepId, input_class: InputClass) -> LeafSpec {
    LeafSpec {
        step_id,
        feature_group: FeatureGroup::WorkspaceSearchModal,
        input_class,
        effect_class: EffectClass::KucRetainedUiEffect,
        source_marker: SourceMarker::AuditRow(AuditRow::WorkspaceModalTabs),
        preconditions: OPEN,
        lifecycle: LifecycleRequirement::new(
            FocusLifecycle::FocusRemainsStable,
            MutationPolicy::NoHostMutationRequired,
        ),
        release_blocker: None,
    }
}

const fn filename_leaf(
    step_id: FullEditorStepId,
    input_class: InputClass,
    effect_class: EffectClass,
    preconditions: &'static [Precondition],
) -> LeafSpec {
    let mutation = match effect_class {
        EffectClass::InProcessHostEffect => MutationPolicy::HostStateMutationExpected,
        _ => MutationPolicy::NoHostMutationRequired,
    };
    LeafSpec {
        step_id,
        feature_group: FeatureGroup::WorkspaceSearchFilename,
        input_class,
        effect_class,
        source_marker: SourceMarker::AuditRow(AuditRow::WorkspaceFilenameFilterFamily),
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
    use std::collections::HashSet;

    #[test]
    fn modal_tabs_are_unique_and_assembled_with_four_evidence() {
        let manifest = FullEditorScenarioManifest::document_find_and_replace_seed();
        let actual = manifest
            .leaves
            .iter()
            .filter(|leaf| matches!(leaf.feature_group, FeatureGroup::WorkspaceSearchModal))
            .collect::<Vec<_>>();
        let ids = actual
            .iter()
            .map(|leaf| leaf.step_id)
            .collect::<HashSet<_>>();
        assert_eq!(actual.len(), 3);
        assert_eq!(ids.len(), 3);
        assert!(
            actual
                .iter()
                .all(|leaf| leaf.required_evidence == required_evidence()
                    && matches!(
                        leaf.source_marker,
                        SourceMarker::AuditRow(AuditRow::WorkspaceModalTabs)
                    ))
        );
    }
}
