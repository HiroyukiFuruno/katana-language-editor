use crate::scenario_manifest_leaf::LeafSpec;
use crate::scenario_manifest_types::{
    AuditRow, EffectClass, FeatureGroup, FocusLifecycle, FullEditorStepId, InputClass,
    LifecycleRequirement, MutationPolicy, Precondition, ReleaseBlocker, SourceMarker,
};

macro_rules! scenario_row {
    ($step_id:ident $group:ident $input:ident $effect:ident audit($audit:ident) $pre:ident $life:ident) => {
        LeafSpec {
            step_id: $step_id,
            feature_group: FeatureGroup::$group,
            input_class: InputClass::$input,
            effect_class: EffectClass::$effect,
            source_marker: SourceMarker::AuditRow(AuditRow::$audit),
            preconditions: $pre,
            lifecycle: $life(),
            release_blocker: NO_BLOCKER,
        }
    };
}

const HOST_DOC: &[Precondition] = &[
    Precondition::HostStarted,
    Precondition::DocumentSearchStateAvailable,
];
const OPEN: &[Precondition] = &[Precondition::SearchBarOpen];
const QUERY: &[Precondition] = &[
    Precondition::SearchBarOpen,
    Precondition::ActiveDocumentAvailable,
    Precondition::QueryCommitted,
];
const EMPTY: &[Precondition] = &[
    Precondition::SearchBarOpen,
    Precondition::ActiveDocumentAvailable,
    Precondition::QueryEmpty,
];
const NO_MATCH: &[Precondition] = &[
    Precondition::SearchBarOpen,
    Precondition::ActiveDocumentAvailable,
    Precondition::QueryHasNoMatches,
];
const MATCH: &[Precondition] = &[Precondition::SearchBarOpen, Precondition::QueryHasMatches];
const NEXT_WRAP: &[Precondition] = &[
    Precondition::SearchBarOpen,
    Precondition::QueryHasMatches,
    Precondition::LastMatchActive,
];
const PREV_WRAP: &[Precondition] = &[
    Precondition::SearchBarOpen,
    Precondition::QueryHasMatches,
    Precondition::FirstMatchActive,
];
const MD_MATCH: &[Precondition] = &[
    Precondition::SearchBarOpen,
    Precondition::MarkdownDocumentActive,
    Precondition::QueryHasMatches,
];
const MD_NO_MATCH: &[Precondition] = &[
    Precondition::SearchBarOpen,
    Precondition::MarkdownDocumentActive,
    Precondition::QueryHasNoMatches,
];
const BOUNDARY: &[Precondition] = &[
    Precondition::WorkspaceSearchAvailable,
    Precondition::WorkspaceSearchUnavailable,
];
const NO_BLOCKER: Option<ReleaseBlocker> = None;

pub(crate) const FIND_OPEN_KEYBOARD: FullEditorStepId = FullEditorStepId::new("FindOpenKeyboard");
pub(crate) const FIND_CLOSE_ESCAPE: FullEditorStepId = FullEditorStepId::new("FindCloseEscape");
pub(crate) const FIND_CLOSE_POINTER: FullEditorStepId = FullEditorStepId::new("FindClosePointer");
pub(crate) const FIND_CLOSE_ACCESS_KIT: FullEditorStepId =
    FullEditorStepId::new("FindCloseAccessKit");
pub(crate) const FIND_QUERY_TEXT: FullEditorStepId = FullEditorStepId::new("FindQueryText");
pub(crate) const FIND_QUERY_IME_COMMIT: FullEditorStepId =
    FullEditorStepId::new("FindQueryImeCommit");
pub(crate) const FIND_QUERY_EMPTY: FullEditorStepId = FullEditorStepId::new("FindQueryEmpty");
pub(crate) const FIND_QUERY_NO_RESULT: FullEditorStepId =
    FullEditorStepId::new("FindQueryNoResult");
pub(crate) const FIND_NEXT_POINTER: FullEditorStepId = FullEditorStepId::new("FindNextPointer");
pub(crate) const FIND_NEXT_KEYBOARD: FullEditorStepId = FullEditorStepId::new("FindNextKeyboard");
pub(crate) const FIND_NEXT_ACCESS_KIT: FullEditorStepId =
    FullEditorStepId::new("FindNextAccessKit");
pub(crate) const FIND_PREV_POINTER: FullEditorStepId = FullEditorStepId::new("FindPrevPointer");
pub(crate) const FIND_PREV_KEYBOARD: FullEditorStepId = FullEditorStepId::new("FindPrevKeyboard");
pub(crate) const FIND_PREV_ACCESS_KIT: FullEditorStepId =
    FullEditorStepId::new("FindPrevAccessKit");
pub(crate) const FIND_NEXT_WRAP: FullEditorStepId = FullEditorStepId::new("FindNextWrap");
pub(crate) const FIND_PREV_WRAP: FullEditorStepId = FullEditorStepId::new("FindPrevWrap");
pub(crate) const FIND_ZERO_MATCH_NAVIGATION: FullEditorStepId =
    FullEditorStepId::new("FindZeroMatchNavigation");
pub(crate) const FIND_FOCUS_RESTORE: FullEditorStepId = FullEditorStepId::new("FindFocusRestore");
pub(crate) const FIND_MARKDOWN_TEXT_HIT: FullEditorStepId =
    FullEditorStepId::new("FindMarkdownTextHit");
pub(crate) const FIND_INLINE_CODE_HIT: FullEditorStepId =
    FullEditorStepId::new("FindInlineCodeHit");
pub(crate) const FIND_URL_EXCLUSION: FullEditorStepId = FullEditorStepId::new("FindUrlExclusion");
pub(crate) const FIND_HTML_EXCLUSION: FullEditorStepId = FullEditorStepId::new("FindHtmlExclusion");
pub(crate) const FIND_WORKSPACE_BOUNDARY: FullEditorStepId =
    FullEditorStepId::new("FindWorkspaceBoundary");

pub(crate) const DOCUMENT_FIND_LEAF_SPECS: &[LeafSpec] = &[
    scenario_row!(FIND_OPEN_KEYBOARD DocumentFind Keyboard KucRetainedUiEffect audit(Open) HOST_DOC focus_search),
    scenario_row!(FIND_CLOSE_ESCAPE DocumentFind Keyboard InProcessHostEffect audit(ToggleClose) OPEN close),
    scenario_row!(FIND_CLOSE_POINTER DocumentFind Pointer InProcessHostEffect audit(ToggleClose) OPEN close),
    scenario_row!(FIND_CLOSE_ACCESS_KIT DocumentFind AccessKit InProcessHostEffect audit(ToggleClose) OPEN close),
    scenario_row!(FIND_QUERY_TEXT DocumentFind TextEdit InProcessHostEffect audit(QueryChange) QUERY matches),
    scenario_row!(FIND_QUERY_IME_COMMIT DocumentFind ImeCommit InProcessHostEffect audit(QueryChange) QUERY matches),
    scenario_row!(FIND_QUERY_EMPTY DocumentFind QueryState NoMutationHostEffect audit(QueryChange) EMPTY no_mutation),
    scenario_row!(FIND_QUERY_NO_RESULT DocumentFind QueryState NoMutationHostEffect audit(QueryChange) NO_MATCH no_mutation),
    scenario_row!(FIND_NEXT_POINTER DocumentFind Pointer NativeExternalHostEffect audit(NextPrevious) MATCH navigation),
    scenario_row!(FIND_NEXT_KEYBOARD DocumentFind Keyboard NativeExternalHostEffect audit(NextPrevious) MATCH navigation),
    scenario_row!(FIND_NEXT_ACCESS_KIT DocumentFind AccessKit NativeExternalHostEffect audit(NextPrevious) MATCH navigation),
    scenario_row!(FIND_PREV_POINTER DocumentFind Pointer NativeExternalHostEffect audit(NextPrevious) MATCH navigation),
    scenario_row!(FIND_PREV_KEYBOARD DocumentFind Keyboard NativeExternalHostEffect audit(NextPrevious) MATCH navigation),
    scenario_row!(FIND_PREV_ACCESS_KIT DocumentFind AccessKit NativeExternalHostEffect audit(NextPrevious) MATCH navigation),
    scenario_row!(FIND_NEXT_WRAP DocumentFind Keyboard NativeExternalHostEffect audit(NextPrevious) NEXT_WRAP navigation),
    scenario_row!(FIND_PREV_WRAP DocumentFind Keyboard NativeExternalHostEffect audit(NextPrevious) PREV_WRAP navigation),
    scenario_row!(FIND_ZERO_MATCH_NAVIGATION DocumentFind QueryState NoMutationHostEffect audit(NextPrevious) NO_MATCH no_mutation),
    scenario_row!(FIND_FOCUS_RESTORE DocumentFind Keyboard InProcessHostEffect audit(ToggleClose) OPEN focus_restore),
    scenario_row!(FIND_MARKDOWN_TEXT_HIT DocumentFindMarkdownMatching QueryState NativeExternalHostEffect audit(MatchComputation) MD_MATCH matches),
    scenario_row!(FIND_INLINE_CODE_HIT DocumentFindMarkdownMatching QueryState NativeExternalHostEffect audit(MatchComputation) MD_MATCH matches),
    scenario_row!(FIND_URL_EXCLUSION DocumentFindMarkdownMatching QueryState NoMutationHostEffect audit(MatchComputation) MD_NO_MATCH no_mutation),
    scenario_row!(FIND_HTML_EXCLUSION DocumentFindMarkdownMatching QueryState NoMutationHostEffect audit(MatchComputation) MD_NO_MATCH no_mutation),
    scenario_row!(FIND_WORKSPACE_BOUNDARY DocumentWorkspaceBoundary HostBoundary NativeExternalHostEffect audit(WorkspaceBoundary) BOUNDARY no_mutation),
];

const fn focus_search() -> LifecycleRequirement {
    LifecycleRequirement::new(
        FocusLifecycle::SearchInputReceivesFocus,
        MutationPolicy::HostStateMutationExpected,
    )
}

const fn close() -> LifecycleRequirement {
    LifecycleRequirement::new(
        FocusLifecycle::EditorFocusRestoredOrPreserved,
        MutationPolicy::MatchStateMutationExpected,
    )
}

const fn focus_restore() -> LifecycleRequirement {
    LifecycleRequirement::new(
        FocusLifecycle::EditorFocusRestoredOrPreserved,
        MutationPolicy::NoHostMutationRequired,
    )
}

const fn matches() -> LifecycleRequirement {
    LifecycleRequirement::new(
        FocusLifecycle::FocusRemainsStable,
        MutationPolicy::MatchStateMutationExpected,
    )
}

const fn navigation() -> LifecycleRequirement {
    LifecycleRequirement::new(
        FocusLifecycle::FocusRemainsStable,
        MutationPolicy::NavigationOnlyMutationExpected,
    )
}

const fn no_mutation() -> LifecycleRequirement {
    LifecycleRequirement::new(
        FocusLifecycle::FocusRemainsStable,
        MutationPolicy::NoHostMutationRequired,
    )
}
