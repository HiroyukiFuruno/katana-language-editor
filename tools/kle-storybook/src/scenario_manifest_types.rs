pub(crate) const FIXED_KATANA_SOURCE_REVISION: &str = "4f6a6287c650a38633c7baeb544a92e739c68567";
pub(crate) const REQUIRED_EVIDENCE_LAYER_COUNT: usize = [(), (), (), ()].len();
pub(crate) type EvidenceStack = [EvidenceRequirement; REQUIRED_EVIDENCE_LAYER_COUNT];

macro_rules! compact_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident { $($variant:ident),+ $(,)? }) => {
        $(#[$meta])* $vis enum $name { $($variant),+ }
    };
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FullEditorScenarioManifest {
    pub(crate) source_revision: SourceRevisionIdentity,
    pub(crate) leaves: Vec<FullEditorScenarioLeaf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FullEditorScenarioLeaf {
    pub(crate) step_id: FullEditorStepId,
    pub(crate) feature_group: FeatureGroup,
    pub(crate) input_class: InputClass,
    pub(crate) effect_class: EffectClass,
    pub(crate) source_marker: SourceMarker,
    pub(crate) preconditions: Vec<Precondition>,
    pub(crate) lifecycle: LifecycleRequirement,
    pub(crate) required_evidence: EvidenceStack,
    pub(crate) release_blocker: Option<ReleaseBlocker>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SourceRevisionIdentity {
    pub(crate) repository: SourceRepository,
    pub(crate) revision: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct FullEditorStepId(&'static str);

impl FullEditorStepId {
    pub(crate) const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub(crate) const fn as_str(&self) -> &'static str {
        self.0
    }
}

const _: fn(&FullEditorStepId) -> &'static str = FullEditorStepId::as_str;

compact_enum!(
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) enum FeatureGroup {
        DocumentFind,
        DocumentFindMarkdownMatching,
        DocumentWorkspaceBoundary,
        WorkspaceSearchModal,
        WorkspaceSearchFilename,
        WorkspaceSearchMarkdown,
        WorkspaceSearchResults,
        AuthoringToolbar,
        ReplaceBlocker,
    }
);
compact_enum!(
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) enum InputClass {
        Keyboard,
        Pointer,
        AccessKit,
        TextEdit,
        ImeCommit,
        QueryState,
        HostBoundary,
        SourceRouteAbsence,
    }
);
compact_enum!(
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) enum EffectClass {
        KucRetainedUiEffect,
        InProcessHostEffect,
        NativeExternalHostEffect,
        NoMutationHostEffect,
        ReleaseBlocker,
    }
);
compact_enum!(
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) enum SourceRepository {
        FixedKatanaSource,
    }
);
compact_enum!(
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) enum AuditRow {
        Open,
        ToggleClose,
        QueryChange,
        MatchComputation,
        NextPrevious,
        WorkspaceBoundary,
        WorkspaceModalTabs,
        WorkspaceFilenameFilterFamily,
        WorkspaceMarkdownHistoryFamily,
        WorkspaceFilenameResultSelect,
        WorkspaceMarkdownResultJump,
        AuthoringInline,
        AuthoringStructure,
        AuthoringReference,
    }
);
compact_enum!(
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) enum FixedSourceRouteAbsent {
        GeneralReplace,
        AllMatches,
    }
);
compact_enum!(
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) enum Precondition {
        HostStarted,
        DocumentSearchStateAvailable,
        SearchBarOpen,
        ActiveDocumentAvailable,
        QueryCommitted,
        QueryEmpty,
        QueryHasMatches,
        QueryHasNoMatches,
        FirstMatchActive,
        LastMatchActive,
        MarkdownDocumentActive,
        WorkspaceSearchAvailable,
        WorkspaceSearchUnavailable,
        WorkspaceSearchOpen,
        FixedSourceRouteAbsent,
    }
);
compact_enum!(
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) enum FocusLifecycle {
        SearchInputReceivesFocus,
        EditorFocusRestoredOrPreserved,
        FocusRemainsStable,
        NotApplicable,
    }
);
compact_enum!(
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) enum MutationPolicy {
        HostStateMutationExpected,
        NavigationOnlyMutationExpected,
        MatchStateMutationExpected,
        NoHostMutationRequired,
        ReleaseBlockedNoLocalMutation,
    }
);
compact_enum!(
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) enum EvidenceRequirement {
        KucRoot,
        AccessKit,
        KleTransit,
        ClassAppropriateEffect,
    }
);
compact_enum!(
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) enum ReleaseBlockerReason {
        FixedSourceRouteAbsent,
    }
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SourceMarker {
    AuditRow(AuditRow),
    FixedSourceRouteAbsent(FixedSourceRouteAbsent),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LifecycleRequirement {
    pub(crate) focus: FocusLifecycle,
    pub(crate) mutation: MutationPolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ReleaseBlocker {
    pub(crate) reason: ReleaseBlockerReason,
    pub(crate) success_possible: bool,
    pub(crate) host_specification_required: bool,
    pub(crate) local_semantics_required: bool,
}
