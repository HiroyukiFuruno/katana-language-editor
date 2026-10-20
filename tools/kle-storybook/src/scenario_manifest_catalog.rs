#[path = "scenario_manifest_authoring_toolbar.rs"]
mod scenario_manifest_authoring_toolbar;
#[path = "scenario_manifest_workspace_search.rs"]
pub(crate) mod scenario_manifest_workspace_search;
#[path = "scenario_manifest_workspace_search_markdown.rs"]
pub(crate) mod scenario_manifest_workspace_search_markdown;
#[path = "scenario_manifest_workspace_search_results.rs"]
mod scenario_manifest_workspace_search_results;

use crate::scenario_manifest_document_find::DOCUMENT_FIND_LEAF_SPECS;
use crate::scenario_manifest_leaf::LeafSpec;
use crate::scenario_manifest_replace::REPLACE_LEAF_SPECS;
use crate::scenario_manifest_types::{
    EffectClass, EvidenceRequirement, EvidenceStack, FeatureGroup, FullEditorScenarioLeaf,
    FullEditorScenarioManifest, InputClass, SourceMarker, SourceRevisionIdentity,
};
use scenario_manifest_authoring_toolbar::AUTHORING_TOOLBAR_LEAF_SPECS;
use scenario_manifest_workspace_search::WORKSPACE_SEARCH_MODAL_TAB_LEAF_SPECS;
use scenario_manifest_workspace_search_markdown::WORKSPACE_SEARCH_MARKDOWN_LEAF_SPECS;
use scenario_manifest_workspace_search_results::WORKSPACE_SEARCH_RESULT_LEAF_SPECS;

pub(crate) type ManifestCompileContract =
    fn(&FullEditorScenarioManifest) -> ManifestCompileContractSummary;
pub(crate) type ManifestCompileContractSummary = (SourceRevisionIdentity, usize, usize);

pub(crate) struct ScenarioManifestCatalog;

impl ScenarioManifestCatalog {
    pub(crate) fn assemble_leaves() -> Vec<FullEditorScenarioLeaf> {
        DOCUMENT_FIND_LEAF_SPECS
            .iter()
            .chain(REPLACE_LEAF_SPECS.iter())
            .chain(WORKSPACE_SEARCH_MODAL_TAB_LEAF_SPECS.iter())
            .chain(WORKSPACE_SEARCH_MARKDOWN_LEAF_SPECS.iter())
            .chain(WORKSPACE_SEARCH_RESULT_LEAF_SPECS.iter())
            .chain(AUTHORING_TOOLBAR_LEAF_SPECS.iter())
            .copied()
            .map(leaf_from_spec)
            .collect()
    }

    pub(crate) fn required_evidence() -> EvidenceStack {
        [
            EvidenceRequirement::KucRoot,
            EvidenceRequirement::AccessKit,
            EvidenceRequirement::KleTransit,
            EvidenceRequirement::ClassAppropriateEffect,
        ]
    }

    pub(crate) fn scenario_manifest_compile_contract(
        manifest: &FullEditorScenarioManifest,
    ) -> ManifestCompileContractSummary {
        manifest.leaves.first().map(touch_leaf_contract_fields);
        (
            manifest.source_revision,
            manifest.leaves.len(),
            manifest
                .leaves
                .iter()
                .filter(|leaf| {
                    leaf.release_blocker
                        .is_some_and(|blocker| !blocker.success_possible)
                })
                .count(),
        )
    }
}

fn leaf_from_spec(spec: LeafSpec) -> FullEditorScenarioLeaf {
    FullEditorScenarioLeaf {
        step_id: spec.step_id,
        feature_group: spec.feature_group,
        input_class: spec.input_class,
        effect_class: spec.effect_class,
        source_marker: spec.source_marker,
        preconditions: spec.preconditions.to_vec(),
        lifecycle: spec.lifecycle,
        required_evidence: ScenarioManifestCatalog::required_evidence(),
        release_blocker: spec.release_blocker,
    }
}

fn touch_leaf_contract_fields(leaf: &FullEditorScenarioLeaf) {
    let _: (
        _,
        FeatureGroup,
        InputClass,
        EffectClass,
        SourceMarker,
        _,
        EvidenceStack,
        _,
    ) = (
        leaf.step_id,
        leaf.feature_group,
        leaf.input_class,
        leaf.effect_class,
        leaf.source_marker,
        leaf.preconditions.as_slice(),
        leaf.required_evidence,
        leaf.release_blocker,
    );
}

#[cfg(test)]
pub(crate) use compatibility::evidence as required_evidence;

#[cfg(test)]
mod compatibility {
    pub(crate) fn evidence() -> super::EvidenceStack {
        super::ScenarioManifestCatalog::required_evidence()
    }
}
