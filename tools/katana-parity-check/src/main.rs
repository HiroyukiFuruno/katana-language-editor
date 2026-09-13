mod baseline;
mod baseline_evidence;
#[cfg(test)]
mod capability_manifest;
mod context_menu_contract;
mod context_menu_contract_declaration;
mod context_menu_contract_entries;
mod context_menu_contract_entries_code;
mod context_menu_contract_entries_root;
mod context_menu_contract_entries_routes;
mod context_menu_contract_entry_types;
mod context_menu_contract_host_e2e;
#[cfg(test)]
mod context_menu_contract_host_tests;
mod context_menu_contract_inventory;
#[cfg(test)]
mod context_menu_contract_tests;
mod context_menu_contract_types;
mod context_menu_contract_validation;
#[cfg(test)]
mod matrix;
mod release_gate;
mod release_gate_dependency;
mod release_gate_kle;
mod release_gate_native_host_workflow;
mod release_gate_publish;
#[cfg(test)]
mod release_gate_publish_tests;
mod release_gate_release_workflow;
mod release_gate_source_closure_artifact;
mod release_gate_source_closure_assemble;
mod release_gate_source_closure_kuc;
#[cfg(test)]
mod release_gate_source_closure_kuc_tests;
mod release_gate_source_closure_provenance;
mod release_gate_source_closure_recipe;
mod release_gate_source_closure_reference;
mod release_gate_sources;
#[cfg(test)]
mod release_gate_tests;
mod release_gate_workflow;
mod requirements;
pub mod source_closure;
mod source_evidence;
#[cfg(test)]
mod source_inventory;
#[cfg(test)]
mod source_inventory_decisive;
#[cfg(test)]
mod source_inventory_doc;
#[cfg(test)]
mod source_inventory_features;
#[cfg(test)]
mod source_inventory_importance;
mod source_inventory_repo;
mod system;
mod user_mandated_replace_route;

use baseline::KatanaBaselineAudit;
use context_menu_contract::ContextMenuContractAudit;
use release_gate::ReleaseGateAudit;
use source_closure::{ArtifactValidationMode, LeafCapabilityAudit};
use user_mandated_replace_route::UserMandatedReplaceRouteAudit;

fn main() -> Result<(), String> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if arguments.first().map(String::as_str) == Some("source-closure") {
        return source_closure::run_cli(&arguments[1..]);
    }
    let mode = parse_validation_mode(&arguments)?;
    let mut failures: Vec<String> = vec![];

    let repo_root = source_inventory_repo::SourceInventoryRepo::resolve_katana_repo();
    run_audit(
        "source-closure artifact audit",
        LeafCapabilityAudit::validate(&repo_root, mode),
        &mut failures,
    );
    if mode == ArtifactValidationMode::Full {
        run_audit(
            "source-closure host E2E execution audit",
            LeafCapabilityAudit::validate_host_e2e_execution(),
            &mut failures,
        );
    }
    run_audit(
        "user-mandated replace source-route audit",
        UserMandatedReplaceRouteAudit::validate(&repo_root),
        &mut failures,
    );
    if mode == ArtifactValidationMode::Full {
        run_audit(
            "context-menu structural leaf contract audit",
            ContextMenuContractAudit::validate(&repo_root),
            &mut failures,
        );
    }
    run_audit(
        "requirements and baseline audit",
        KatanaBaselineAudit::validate(),
        &mut failures,
    );
    run_audit(
        "release gate audit",
        ReleaseGateAudit::validate(),
        &mut failures,
    );
    if !failures.is_empty() {
        return Err(format!(
            "katana parity verification failed with {} blocker items:\n{}",
            failures.len(),
            failures.join("\n")
        ));
    }

    println!("katana {mode:?} parity verification completed through source closure");
    Ok(())
}

fn parse_validation_mode(arguments: &[String]) -> Result<ArtifactValidationMode, String> {
    match arguments {
        [] => Ok(ArtifactValidationMode::KleRelease),
        [flag, value] if flag == "--mode" && value == "rc" => Ok(ArtifactValidationMode::Rc),
        [flag, value] if flag == "--mode" && value == "kle-release" => Ok(ArtifactValidationMode::KleRelease),
        [flag, value] if flag == "--mode" && value == "downstream-full" => Ok(ArtifactValidationMode::Full),
        [flag, value] if flag == "--mode" && value == "full" => Ok(ArtifactValidationMode::Full),
        _ => Err(
            "usage: katana-parity-check [--mode rc|kle-release|downstream-full|full] or katana-parity-check source-closure …"
                .to_string(),
        ),
    }
}

fn run_audit(name: &str, result: Result<(), String>, failures: &mut Vec<String>) {
    if let Err(error) = result {
        failures.push(format!("{name}: {error}"));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parity_entrypoint_uses_only_source_closure_for_leaf_acceptance() -> Result<(), String> {
        let source = include_str!("main.rs");
        let file = syn::parse_file(source)
            .map_err(|error| format!("main.rs must remain valid Rust: {error}"))?;
        let declared_modules = file
            .items
            .iter()
            .filter_map(|item| match item {
                syn::Item::Mod(module) => Some(module.ident.to_string()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(source.contains("LeafCapabilityAudit::validate(&repo_root, mode)"));
        assert!(source.contains("LeafCapabilityAudit::validate_host_e2e_execution"));
        assert!(source.contains("if mode == ArtifactValidationMode::Full"));
        assert!(!source.contains(&["FEAT", "URES"].concat()));
        assert!(!source.contains(&["CapabilityManifest", "Audit"].concat()));
        assert!(!source.contains(&["LegacyLeafCapability", "Audit"].concat()));
        assert!(!source.contains(&["for ", "feature in"].concat()));
        assert!(!source.contains(&["validate_", "feature("].concat()));
        assert!(source.contains("ReleaseGateAudit::validate()"));
        assert!(
            declared_modules
                .iter()
                .any(|declared| declared == "release_gate")
        );
        assert!(
            declared_modules
                .iter()
                .any(|declared| declared == "source_closure")
        );
        Ok(())
    }

    #[test]
    fn parity_entrypoint_rejects_unknown_validation_modes() {
        let arguments = ["--mode".to_string(), "skip".to_string()];
        let result = super::parse_validation_mode(&arguments);
        assert!(matches!(result, Err(message) if message.contains("usage:")));
    }

    #[test]
    fn parity_entrypoint_defaults_to_kle_release_and_keeps_downstream_aliases() {
        assert_eq!(
            super::parse_validation_mode(&[]),
            Ok(super::ArtifactValidationMode::KleRelease)
        );
        assert_eq!(
            super::parse_validation_mode(&["--mode".to_string(), "downstream-full".to_string()]),
            Ok(super::ArtifactValidationMode::Full)
        );
        assert_eq!(
            super::parse_validation_mode(&["--mode".to_string(), "full".to_string()]),
            Ok(super::ArtifactValidationMode::Full)
        );
    }
}
