use crate::matrix::EvidenceClassification;

pub(crate) struct CapabilityManifest {
    pub(crate) classification: EvidenceClassification,
    pub(crate) katana_sources: &'static [KatanaSourceEvidence],
    pub(crate) owners: &'static [CapabilityOwner],
    pub(crate) kle_actual_input: KleActualInputEvidence,
    pub(crate) host_e2e: KleHostE2eEvidence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CapabilityOwner {
    KucRuntime,
    KleBinding,
    KatanaHost,
}

#[derive(Clone, Copy)]
pub(crate) struct KatanaSourceEvidence {
    pub(crate) path: &'static str,
    pub(crate) line: usize,
    pub(crate) marker: &'static str,
}

pub(crate) struct KleActualInputEvidence {
    pub(crate) feature_id: &'static str,
    pub(crate) source_path: &'static str,
    pub(crate) selector: &'static str,
    pub(crate) harness: KleActualFrameHarness,
}

#[derive(Clone, Copy)]
pub(crate) struct KleActualFrameHarness {
    pub(crate) public_show_callsite: KleSourceLocator,
    pub(crate) raw_input_root: KleSourceLocator,
    pub(crate) raw_input_construction: KleSourceLocator,
    pub(crate) scenario_implementation: KleSourceLocator,
    pub(crate) symbol: &'static str,
    pub(crate) call_path: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct KleSourceLocator {
    pub(crate) source_path: &'static str,
    pub(crate) line: usize,
    pub(crate) marker: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct KleHostE2eEvidence {
    pub(crate) test: KleHostE2eTestLocator,
    pub(crate) effect: HostEffectKind,
}

#[derive(Clone, Copy)]
pub(crate) struct KleHostE2eTestLocator {
    pub(crate) target: &'static str,
    pub(crate) source_path: &'static str,
    pub(crate) selector: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HostEffectKind {
    ContextAuthoring,
    ToolbarAuthoring,
    Save,
    Format,
    ExternalIngestFailClosed,
    Missing,
}

impl HostEffectKind {
    pub(crate) const fn description(self) -> &'static str {
        match self {
            Self::ContextAuthoring => "context authoring document effect",
            Self::ToolbarAuthoring => "toolbar authoring document effect",
            Self::Save => "save persistence and dirty-state effect",
            Self::Format => "format persistence and document effect",
            Self::ExternalIngestFailClosed => "external ingest fail-closed document effect",
            Self::Missing => "missing actual host-effect coverage",
        }
    }
}
