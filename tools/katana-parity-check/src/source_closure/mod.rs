mod action_origins;
mod actual_reference;
pub mod artifact_model;
mod artifact_paths;
mod artifact_validation_mode;
mod artifact_validator;
mod ast_resolution;
mod ast_scan;
mod ast_visit;
mod branch_catalog;
mod canonical_leaf_binding;
mod canonical_leaf_projection;
mod context_menu_manifest;
mod edge_model;
mod execution_coverage;
mod execution_validation;
mod external_ui_dependency;
mod fingerprint;
mod fixed_reference;
mod generator;
mod input_validation;
mod kle_release_evidence;
mod kle_release_evidence_integrity;
mod kle_release_gate;
mod leaf_audit;
mod leaf_host_e2e_validation;
#[cfg(test)]
pub mod leaf_join;
mod leaf_validation;
mod lexical_resolver;
mod load;
mod materializer;
mod model;
mod native_target;
mod operational_capture_support;
mod operational_cfg;
mod operational_cfg_predicate;
mod operational_cli;
mod operational_evidence;
mod operational_evidence_validation;
#[cfg(test)]
mod operational_fixture_write;
mod operational_identity;
mod operational_input;
mod operational_input_assembly;
mod operational_lifecycle;
mod operational_loader;
mod operational_output;
mod operational_paths;
mod operational_process;
mod operational_profile;
mod operational_profile_capture;
mod operational_provenance_capture;
mod operational_provenance_generator;
mod operational_provenance_requirements;
mod operational_provenance_source_universe;
mod operational_publication;
mod operational_publication_copy;
mod operational_publication_fingerprint;
mod operational_publication_materialized;
mod operational_repository_provenance;
mod operational_repository_validation;
mod operational_root;
mod operational_staging;
mod operational_staging_layout;
mod operational_staging_receipt;
mod operational_time;
mod operational_validation;
mod path_resolution;
mod profile_model;
mod profile_validation;
#[cfg(test)]
mod qualified_action_tests;
pub(crate) mod requirement_binding;
mod requirement_diagnostic;
pub(crate) mod requirement_source_inventory;
mod requirement_source_inventory_build;
mod requirement_source_inventory_scan;
#[cfg(test)]
mod requirement_source_inventory_tests;
mod root_model;
mod root_validation;
mod scan_state;
mod source_action_binding;
pub(crate) mod source_requirement_alias_ledger;
#[cfg(test)]
mod source_requirement_alias_ledger_tests;
mod source_requirement_alias_parser;
mod source_requirement_alias_validation;
pub(crate) mod source_requirements_ledger;
#[cfg(test)]
mod source_requirements_ledger_tests;
mod source_roots;
mod source_roots_ledger;
#[cfg(test)]
mod source_roots_ledger_contract_tests;
#[cfg(test)]
mod source_roots_tests;
mod source_validation;
mod storybook_media_validation;
mod structured_leaf_candidates;
mod user_mandated_extensions;
mod validator;
mod validator_helpers;

pub use actual_reference::{ActualReferenceBinding, ActualReferenceVerifier};
pub use artifact_paths::ArtifactPaths;
pub use artifact_validator::{ArtifactValidationMode, SourceClosureArtifactValidator};
pub use generator::LeafCapabilityAudit;
pub use materializer::SourceClosureMaterializer;
pub use operational_input::{
    EvidenceRef, InactiveCfgEdge, KatanaTreeEvidence, ProfileProbeInput, RootEvidence,
    RootProvenance, SourceClosureInput,
};
pub use operational_loader::{SourceClosureInputLoaderVerifier, VerifiedSourceClosureInput};
pub use root_model::ManifestRoot;

pub(super) fn run_cli(args: &[String]) -> Result<(), String> {
    operational_lifecycle::run_cli(args)
}
