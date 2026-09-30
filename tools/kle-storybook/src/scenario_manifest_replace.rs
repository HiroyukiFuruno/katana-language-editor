use crate::scenario_manifest_leaf::LeafSpec;
use crate::scenario_manifest_types::{
    EffectClass, FeatureGroup, FixedSourceRouteAbsent, FocusLifecycle, FullEditorStepId,
    InputClass, LifecycleRequirement, MutationPolicy, Precondition, ReleaseBlocker,
    ReleaseBlockerReason, SourceMarker,
};

macro_rules! scenario_row {
    ($step_id:ident $group:ident $input:ident $effect:ident absent($absent:ident) $pre:ident $life:ident) => {
        LeafSpec {
            step_id: $step_id,
            feature_group: FeatureGroup::$group,
            input_class: InputClass::$input,
            effect_class: EffectClass::$effect,
            source_marker: SourceMarker::FixedSourceRouteAbsent(FixedSourceRouteAbsent::$absent),
            preconditions: $pre,
            lifecycle: $life(),
            release_blocker: BLOCKER,
        }
    };
}

const ABSENT: &[Precondition] = &[Precondition::FixedSourceRouteAbsent];
const BLOCKER: Option<ReleaseBlocker> = Some(ReleaseBlocker {
    reason: ReleaseBlockerReason::FixedSourceRouteAbsent,
    success_possible: false,
    host_specification_required: true,
    local_semantics_required: false,
});
pub(crate) const REPLACE_RELEASE_BLOCKER_GENERAL: FullEditorStepId =
    FullEditorStepId::new("ReplaceReleaseBlocker");
pub(crate) const REPLACE_ALL_RELEASE_BLOCKER: FullEditorStepId =
    FullEditorStepId::new("ReplaceAllReleaseBlocker");
pub(crate) const REPLACE_LEAF_SPECS: &[LeafSpec] = &[
    scenario_row!(REPLACE_RELEASE_BLOCKER_GENERAL ReplaceBlocker SourceRouteAbsence ReleaseBlocker absent(GeneralReplace) ABSENT blocked),
    scenario_row!(REPLACE_ALL_RELEASE_BLOCKER ReplaceBlocker SourceRouteAbsence ReleaseBlocker absent(AllMatches) ABSENT blocked),
];

const fn blocked() -> LifecycleRequirement {
    LifecycleRequirement::new(
        FocusLifecycle::NotApplicable,
        MutationPolicy::ReleaseBlockedNoLocalMutation,
    )
}
