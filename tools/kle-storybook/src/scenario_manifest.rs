use crate::scenario_manifest_catalog::{ManifestCompileContract, ScenarioManifestCatalog};

pub(crate) use crate::scenario_manifest_types::{
    FIXED_KATANA_SOURCE_REVISION, FocusLifecycle, FullEditorScenarioManifest, LifecycleRequirement,
    MutationPolicy, SourceRepository, SourceRevisionIdentity,
};

type ManifestSeedFactory = fn() -> FullEditorScenarioManifest;

const _: ManifestSeedFactory = FullEditorScenarioManifest::document_find_and_replace_seed;
const _: ManifestCompileContract = ScenarioManifestCatalog::scenario_manifest_compile_contract;

impl FullEditorScenarioManifest {
    pub(crate) fn document_find_and_replace_seed() -> Self {
        Self {
            source_revision: SourceRevisionIdentity::fixed_katana_source(),
            leaves: ScenarioManifestCatalog::assemble_leaves(),
        }
    }
}

impl SourceRevisionIdentity {
    pub(crate) fn fixed_katana_source() -> Self {
        Self {
            repository: SourceRepository::FixedKatanaSource,
            revision: FIXED_KATANA_SOURCE_REVISION,
        }
    }
}

impl LifecycleRequirement {
    pub(crate) const fn new(focus: FocusLifecycle, mutation: MutationPolicy) -> Self {
        Self { focus, mutation }
    }
}
