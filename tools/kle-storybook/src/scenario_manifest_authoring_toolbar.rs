use crate::scenario_manifest_leaf::LeafSpec;
use crate::scenario_manifest_types::EffectClass::InProcessHostEffect as InProcess;
use crate::scenario_manifest_types::InputClass::Pointer;
use crate::scenario_manifest_types::Precondition::{ActiveDocumentAvailable, HostStarted};
use crate::scenario_manifest_types::{
    AuditRow, FeatureGroup, FocusLifecycle, FullEditorStepId, LifecycleRequirement, MutationPolicy,
    Precondition, SourceMarker,
};

const ACTIVE_HOST_DOCUMENT: &[Precondition] = &[HostStarted, ActiveDocumentAvailable];

macro_rules! authoring_step {
    ($name:ident, $suffix:literal) => {
        pub(crate) const $name: FullEditorStepId =
            FullEditorStepId::new(concat!("authoring.toolbar.", $suffix));
    };
}

authoring_step!(AUTHORING_TOOLBAR_BOLD, "bold");
authoring_step!(AUTHORING_TOOLBAR_ITALIC, "italic");
authoring_step!(AUTHORING_TOOLBAR_STRIKE, "strike");
authoring_step!(AUTHORING_TOOLBAR_INLINE_CODE, "inline-code");
authoring_step!(AUTHORING_TOOLBAR_HEADING1, "heading1");
authoring_step!(AUTHORING_TOOLBAR_HEADING2, "heading2");
authoring_step!(AUTHORING_TOOLBAR_HEADING3, "heading3");
authoring_step!(AUTHORING_TOOLBAR_BULLET_LIST, "bullet-list");
authoring_step!(AUTHORING_TOOLBAR_NUMBERED_LIST, "numbered-list");
authoring_step!(AUTHORING_TOOLBAR_BLOCKQUOTE, "blockquote");
authoring_step!(AUTHORING_TOOLBAR_HORIZONTAL_RULE, "horizontal-rule");
authoring_step!(AUTHORING_TOOLBAR_CODE_BLOCK, "code-block");
authoring_step!(AUTHORING_TOOLBAR_INSERT_LINK, "insert-link");
authoring_step!(AUTHORING_TOOLBAR_INSERT_TABLE, "insert-table");

pub(crate) const AUTHORING_TOOLBAR_LEAF_SPECS: &[LeafSpec] = &[
    leaf(AUTHORING_TOOLBAR_BOLD, AuditRow::AuthoringInline),
    leaf(AUTHORING_TOOLBAR_ITALIC, AuditRow::AuthoringInline),
    leaf(AUTHORING_TOOLBAR_STRIKE, AuditRow::AuthoringInline),
    leaf(AUTHORING_TOOLBAR_INLINE_CODE, AuditRow::AuthoringInline),
    leaf(AUTHORING_TOOLBAR_HEADING1, AuditRow::AuthoringStructure),
    leaf(AUTHORING_TOOLBAR_HEADING2, AuditRow::AuthoringStructure),
    leaf(AUTHORING_TOOLBAR_HEADING3, AuditRow::AuthoringStructure),
    leaf(AUTHORING_TOOLBAR_BULLET_LIST, AuditRow::AuthoringStructure),
    leaf(
        AUTHORING_TOOLBAR_NUMBERED_LIST,
        AuditRow::AuthoringStructure,
    ),
    leaf(AUTHORING_TOOLBAR_BLOCKQUOTE, AuditRow::AuthoringStructure),
    leaf(
        AUTHORING_TOOLBAR_HORIZONTAL_RULE,
        AuditRow::AuthoringStructure,
    ),
    leaf(AUTHORING_TOOLBAR_CODE_BLOCK, AuditRow::AuthoringStructure),
    leaf(AUTHORING_TOOLBAR_INSERT_LINK, AuditRow::AuthoringReference),
    leaf(AUTHORING_TOOLBAR_INSERT_TABLE, AuditRow::AuthoringReference),
];

const fn leaf(step_id: FullEditorStepId, audit_row: AuditRow) -> LeafSpec {
    LeafSpec {
        step_id,
        feature_group: FeatureGroup::AuthoringToolbar,
        input_class: Pointer,
        effect_class: InProcess,
        source_marker: SourceMarker::AuditRow(audit_row),
        preconditions: ACTIVE_HOST_DOCUMENT,
        lifecycle: LifecycleRequirement::new(
            FocusLifecycle::FocusRemainsStable,
            MutationPolicy::HostStateMutationExpected,
        ),
        release_blocker: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario_manifest::FullEditorScenarioManifest;
    use crate::scenario_manifest_catalog::required_evidence;
    use crate::scenario_manifest_types::{EffectClass, FullEditorScenarioLeaf, InputClass};
    use std::collections::HashSet;

    const SEARCH_GROUPS: &[FeatureGroup] = &[
        FeatureGroup::DocumentFind,
        FeatureGroup::DocumentFindMarkdownMatching,
        FeatureGroup::DocumentWorkspaceBoundary,
        FeatureGroup::WorkspaceSearchModal,
        FeatureGroup::WorkspaceSearchFilename,
        FeatureGroup::WorkspaceSearchMarkdown,
        FeatureGroup::WorkspaceSearchResults,
    ];

    #[test]
    fn authoring_toolbar_registers_fourteen_basic_trigger_leaves() {
        let leaves = authoring_leaves();
        assert_eq!(leaves.len(), [(); 14].len());
        assert_eq!(
            count_marker(&leaves, AuditRow::AuthoringInline),
            [(); 4].len()
        );
        assert_eq!(
            count_marker(&leaves, AuditRow::AuthoringStructure),
            [(); 8].len()
        );
        assert_eq!(
            count_marker(&leaves, AuditRow::AuthoringReference),
            [(); 2].len()
        );
    }

    #[test]
    fn authoring_toolbar_leaves_are_host_only_pointer_triggers_with_four_evidence() {
        for leaf in authoring_leaves() {
            assert!(leaf.step_id.as_str().starts_with("authoring.toolbar."));
            assert_eq!(leaf.input_class, InputClass::Pointer);
            assert_eq!(leaf.effect_class, EffectClass::InProcessHostEffect);
            assert_eq!(leaf.preconditions.as_slice(), ACTIVE_HOST_DOCUMENT);
            assert_eq!(leaf.required_evidence, required_evidence());
            assert_eq!(leaf.required_evidence.len(), [(); 4].len());
            assert!(leaf.release_blocker.is_none());
        }
    }

    #[test]
    fn authoring_toolbar_ids_are_disjoint_from_document_and_workspace_search() {
        let manifest = FullEditorScenarioManifest::document_find_and_replace_seed();
        let authoring = ids_for(&manifest, &[FeatureGroup::AuthoringToolbar]);
        let search = ids_for(&manifest, SEARCH_GROUPS);
        assert_eq!(authoring.len(), [(); 14].len());
        assert!(authoring.is_disjoint(&search));
    }

    fn authoring_leaves() -> Vec<FullEditorScenarioLeaf> {
        FullEditorScenarioManifest::document_find_and_replace_seed()
            .leaves
            .into_iter()
            .filter(|leaf| leaf.feature_group == FeatureGroup::AuthoringToolbar)
            .collect()
    }

    fn ids_for(
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

    fn count_marker(leaves: &[FullEditorScenarioLeaf], row: AuditRow) -> usize {
        leaves
            .iter()
            .filter(|leaf| leaf.source_marker == SourceMarker::AuditRow(row))
            .count()
    }
}
