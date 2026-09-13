use std::path::Path;

use crate::{
    capability_manifest::{HostEffectKind, KleHostE2eEvidence},
    source_inventory_repo::SourceInventoryRepo,
};

use super::{
    capability_manifest_host_e2e_effect::HostEffectContractValidator,
    capability_manifest_host_e2e_runtime::HostRuntimeRouteValidator,
    capability_manifest_host_e2e_syntax::ReachableEvidence,
};

const HOST_E2E_MANIFEST: &str = "tools/katana-host-e2e/Cargo.toml";
const HOST_E2E_PACKAGE_NAME: &str = "name = \"katana-host-e2e\"";
const REJECTED_HOST_EVIDENCE: &[&str] = &[
    "Storybook",
    "AppAction::UpdateBuffer",
    "simulated_host_state",
];

pub(crate) struct HostE2eEvidenceValidator;

impl HostE2eEvidenceValidator {
    pub(crate) fn validate(
        workspace_root: &Path,
        evidence: &KleHostE2eEvidence,
    ) -> Result<(), String> {
        Self::validate_declaration(evidence)?;
        if evidence.effect == HostEffectKind::Missing {
            return Err(format!(
                "missing actual KLE host-effect E2E coverage: {} target {} selector {}",
                evidence.effect.description(),
                evidence.test.target,
                evidence.test.selector
            ));
        }
        let file = Self::read_target_file(workspace_root, evidence)?;
        let test = Self::find_test(&file, evidence)?;
        let reachable = ReachableEvidence::collect(&file, test);
        Self::validate_real_host_route(&reachable, evidence)?;
        HostRuntimeRouteValidator::validate(workspace_root)?;
        HostEffectContractValidator::validate(&reachable, evidence.effect)
    }

    fn validate_declaration(evidence: &KleHostE2eEvidence) -> Result<(), String> {
        if evidence.test.target.is_empty()
            || evidence.test.source_path.is_empty()
            || evidence.test.selector.is_empty()
        {
            return Err("capability manifest has incomplete KLE host E2E evidence".to_string());
        }
        Self::validate_target_name(evidence)
    }

    fn validate_target_name(evidence: &KleHostE2eEvidence) -> Result<(), String> {
        let target = evidence.test.target;
        if target.contains('/')
            || target.contains('.')
            || !target
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            return Err(format!("invalid KLE host E2E Cargo test target: {target}"));
        }
        let expected = format!("tools/katana-host-e2e/tests/{target}.rs");
        (evidence.test.source_path == expected).then_some(()).ok_or_else(|| {
            format!(
                "KLE host E2E source must be the exact Cargo integration target {expected}: got {}",
                evidence.test.source_path
            )
        })
    }

    fn read_target_file(
        workspace_root: &Path,
        evidence: &KleHostE2eEvidence,
    ) -> Result<syn::File, String> {
        let manifest = SourceInventoryRepo::read_file(&workspace_root.join(HOST_E2E_MANIFEST))
            .map_err(|_| format!("missing KLE host E2E manifest: {HOST_E2E_MANIFEST}"))?;
        if !manifest.contains(HOST_E2E_PACKAGE_NAME) {
            return Err("KLE host E2E manifest is not katana-host-e2e".to_string());
        }
        let source =
            SourceInventoryRepo::read_file(&workspace_root.join(evidence.test.source_path))
                .map_err(|_| {
                    format!(
                        "missing KLE host E2E test target source: {}",
                        evidence.test.source_path
                    )
                })?;
        syn::parse_file(&source).map_err(|error| {
            format!(
                "invalid KLE host E2E test source {}: {error}",
                evidence.test.source_path
            )
        })
    }

    fn find_test<'a>(
        file: &'a syn::File,
        evidence: &KleHostE2eEvidence,
    ) -> Result<&'a syn::ItemFn, String> {
        file.items
            .iter()
            .find_map(|item| match item {
                syn::Item::Fn(function)
                    if function.sig.ident == evidence.test.selector
                        && function
                            .attrs
                            .iter()
                            .any(|attribute| attribute.path().is_ident("test")) =>
                {
                    Some(function)
                }
                _ => None,
            })
            .ok_or_else(|| {
                format!(
                    "missing actual KLE host-effect E2E selector: {}::{}",
                    evidence.test.target, evidence.test.selector
                )
            })
    }

    fn validate_real_host_route(
        source: &ReachableEvidence,
        evidence: &KleHostE2eEvidence,
    ) -> Result<(), String> {
        if !Self::has_editor_request_route(source) {
            return Err(format!(
                "KLE host E2E {}::{} does not prove required actual-host route marker KatanaHost::run_editor_request...",
                evidence.test.target, evidence.test.selector
            ));
        }
        for required in ["katana_ui::shell::KatanaApp"] {
            if !source.path_starts_with(required) {
                return Err(format!(
                    "KLE host E2E {}::{} does not prove required actual-host route marker {}",
                    evidence.test.target, evidence.test.selector, required
                ));
            }
        }
        for forbidden in REJECTED_HOST_EVIDENCE {
            if source.path_starts_with(forbidden) || source.string_literals.contains(*forbidden) {
                return Err(format!(
                    "KLE host E2E {}::{} uses rejected non-host evidence: {}",
                    evidence.test.target, evidence.test.selector, forbidden
                ));
            }
        }
        Ok(())
    }

    fn has_editor_request_route(source: &ReachableEvidence) -> bool {
        source.path_starts_with("KatanaHost::run_editor_request")
            || source.path_starts_with("KatanaHost::run_editor_request_with_context")
    }
}
