#[path = "capability_manifest_call_analysis.rs"]
mod capability_manifest_call_analysis;
#[path = "capability_manifest_declaration.rs"]
mod capability_manifest_declaration;
#[path = "capability_manifest_harness.rs"]
mod capability_manifest_harness;
#[path = "capability_manifest_harness_method_chain.rs"]
mod capability_manifest_harness_method_chain;
#[path = "capability_manifest_host_e2e_effect.rs"]
mod capability_manifest_host_e2e_effect;
#[path = "capability_manifest_host_e2e_evidence.rs"]
mod capability_manifest_host_e2e_evidence;
#[path = "capability_manifest_host_e2e_execution.rs"]
mod capability_manifest_host_e2e_execution;
#[path = "capability_manifest_host_e2e_runtime.rs"]
mod capability_manifest_host_e2e_runtime;
#[path = "capability_manifest_host_e2e_syntax.rs"]
mod capability_manifest_host_e2e_syntax;
#[path = "capability_manifest_katana_evidence.rs"]
mod capability_manifest_katana_evidence;
#[path = "capability_manifest_kle_evidence.rs"]
mod capability_manifest_kle_evidence;
#[path = "capability_manifest_types.rs"]
mod capability_manifest_types;

#[cfg(test)]
#[path = "capability_manifest_host_e2e_tests.rs"]
mod capability_manifest_host_e2e_tests;
#[cfg(test)]
#[path = "capability_manifest_test_fixtures.rs"]
pub(crate) mod capability_manifest_test_fixtures;
#[cfg(test)]
#[path = "capability_manifest_test_harness.rs"]
mod capability_manifest_test_harness;
#[cfg(test)]
#[path = "capability_manifest_test_validation.rs"]
mod capability_manifest_test_validation;

use std::path::Path;

use crate::matrix::FeatureVerification;
use crate::source_inventory_repo::SourceInventoryRepo;
use capability_manifest_declaration::DeclarationValidator;
use capability_manifest_harness::HarnessResolver;
use capability_manifest_host_e2e_evidence::HostE2eEvidenceValidator;
use capability_manifest_host_e2e_execution::HostE2eExecutionAudit;
use capability_manifest_katana_evidence::KatanaEvidenceValidator;
use capability_manifest_kle_evidence::KleEvidenceValidator;
pub(crate) use capability_manifest_types::{
    CapabilityManifest, CapabilityOwner, HostEffectKind, KatanaSourceEvidence,
    KleActualFrameHarness, KleActualInputEvidence, KleHostE2eEvidence, KleHostE2eTestLocator,
    KleSourceLocator,
};

pub(crate) struct CapabilityManifestAudit;

impl CapabilityManifestAudit {
    pub(crate) fn validate(feature: &FeatureVerification, repo_root: &Path) -> Result<(), String> {
        let manifest = feature.manifest;
        DeclarationValidator::validate(feature)?;
        let mut failures = Vec::new();
        for source in manifest.katana_sources {
            Self::collect_failure(
                &mut failures,
                KatanaEvidenceValidator::validate_source(repo_root, source),
            );
        }
        Self::collect_failure(
            &mut failures,
            KleEvidenceValidator::validate(&manifest.kle_actual_input),
        );
        match Self::workspace_root() {
            Ok(workspace_root) => Self::collect_failure(
                &mut failures,
                HostE2eEvidenceValidator::validate(&workspace_root, &manifest.host_e2e),
            ),
            Err(error) => failures.push(error),
        }
        failures
            .is_empty()
            .then_some(())
            .ok_or_else(|| failures.join("; "))
    }

    pub(crate) fn validate_feature_set(features: &[FeatureVerification]) -> Result<(), String> {
        for feature in features {
            DeclarationValidator::validate(feature)?;
        }
        DeclarationValidator::validate_distinct_selectors(features)
    }

    pub(crate) fn describe_kle_actual_input(evidence: &KleActualInputEvidence) -> String {
        let workspace_root = match Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
        {
            Ok(root) => root,
            Err(error) => return format!("unresolved KLE workspace root: {error}"),
        };
        let source =
            match SourceInventoryRepo::read_file(&workspace_root.join(evidence.source_path)) {
                Ok(source) => source,
                Err(_) => {
                    return format!(
                        "unresolved KLE actual-input source: {}",
                        evidence.source_path
                    );
                }
            };
        let Some(selector) = KleEvidenceValidator::find_test_function(&source, evidence.selector)
        else {
            return format!(
                "unresolved KLE actual-input selector: {}::{}",
                evidence.source_path, evidence.selector
            );
        };
        match HarnessResolver::resolve(&workspace_root, evidence, selector) {
            Ok(chain) => HarnessResolver::format_chain(evidence.harness.symbol, &chain),
            Err(error) => format!("unresolved: {error}"),
        }
    }

    pub(crate) fn validate_actual_input_at(
        workspace_root: &Path,
        evidence: &KleActualInputEvidence,
    ) -> Result<(), String> {
        KleEvidenceValidator::validate_at(workspace_root, evidence)
    }

    pub(crate) fn validate_host_e2e_at(
        workspace_root: &Path,
        evidence: &KleHostE2eEvidence,
    ) -> Result<(), String> {
        HostE2eEvidenceValidator::validate(workspace_root, evidence)
    }

    pub(crate) fn validate_host_e2e_execution(
        evidence: &[KleHostE2eEvidence],
    ) -> Result<(), String> {
        HostE2eExecutionAudit::validate(&Self::workspace_root()?, evidence)
    }

    pub(crate) fn describe_host_e2e(evidence: &KleHostE2eEvidence) -> String {
        format!(
            "KLE host E2E target {} source {} selector {} expected {}",
            evidence.test.target,
            evidence.test.source_path,
            evidence.test.selector,
            evidence.effect.description(),
        )
    }

    fn workspace_root() -> Result<std::path::PathBuf, String> {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .map_err(|error| format!("failed to resolve KLE workspace root: {error}"))
    }

    pub(super) fn collect_failure(failures: &mut Vec<String>, result: Result<(), String>) {
        if let Err(error) = result {
            failures.push(error);
        }
    }
}

#[cfg(test)]
mod historical_linkage {
    use super::*;

    #[test]
    fn keeps_non_acceptance_analysis_symbols_reachable_for_regression_tests() {
        let _ = CapabilityManifestAudit::validate_actual_input_at
            as fn(&Path, &KleActualInputEvidence) -> Result<(), String>;
        let _ = CapabilityManifestAudit::validate_host_e2e_at
            as fn(&Path, &KleHostE2eEvidence) -> Result<(), String>;
        let _ = CapabilityManifestAudit::validate_host_e2e_execution
            as fn(&[KleHostE2eEvidence]) -> Result<(), String>;
        let _ = CapabilityManifestAudit::describe_host_e2e as fn(&KleHostE2eEvidence) -> String;
        let _ = [
            HostEffectKind::ToolbarAuthoring,
            HostEffectKind::Save,
            HostEffectKind::Format,
        ];
    }
}
