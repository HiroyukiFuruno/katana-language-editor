use crate::scenario_manifest_types::{
    EffectClass, FeatureGroup, FullEditorStepId, InputClass, LifecycleRequirement, Precondition,
    ReleaseBlocker, SourceMarker,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LeafSpec {
    pub(crate) step_id: FullEditorStepId,
    pub(crate) feature_group: FeatureGroup,
    pub(crate) input_class: InputClass,
    pub(crate) effect_class: EffectClass,
    pub(crate) source_marker: SourceMarker,
    pub(crate) preconditions: &'static [Precondition],
    pub(crate) lifecycle: LifecycleRequirement,
    pub(crate) release_blocker: Option<ReleaseBlocker>,
}
