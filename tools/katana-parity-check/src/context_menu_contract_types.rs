use crate::source_evidence::KatanaSourceEvidence;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ContextEvidenceKind {
    ActualHostE2e,
    #[cfg(test)]
    SourceOnly,
    #[cfg(test)]
    StorybookOnly,
    #[cfg(test)]
    SimulatorOnly,
}

#[derive(Clone, Copy)]
pub(crate) struct ContextMenuContractLeaf {
    pub(crate) id: &'static str,
    pub(crate) parent_path: &'static str,
    pub(crate) root_slot: Option<usize>,
    pub(crate) source: KatanaSourceEvidence,
    pub(crate) evidence_kind: ContextEvidenceKind,
    pub(crate) host_e2e_test_source_path: &'static str,
    pub(crate) host_e2e_runtime_source_path: &'static str,
}
