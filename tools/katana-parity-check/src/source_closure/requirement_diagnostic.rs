use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

use serde::Serialize;

mod checkout;
mod output;
mod root;
mod source_scan;

use checkout::{read_and_verify, verify_checkout, verify_fixed_blob};
use output::{validate_output_location, write_new_json};
use root::diagnostic_root;
use source_scan::scan_source;

use super::action_origins::materialize_action_origins;
use super::branch_catalog::materialize_branch_catalog;
use super::fingerprint::sha256_hex;
use super::model::{BranchCatalogArtifact, SourceClosureArtifact, SourceClosureFile};
use super::operational_cli::CliOptions;
use super::operational_input::FIXED_KATANA_REVISION;
use super::requirement_binding::RequirementBindingResult;
use super::requirement_source_inventory::RequirementSourceInventoryReport;
use super::scan_state::ScanState;
use super::source_action_binding::SourceActionBindingReport;
use super::source_requirement_alias_ledger::{
    ALIAS_LEDGER, REQUIREMENTS, ROOT_MANIFEST, RequirementSourceAliasLedger, SOURCE_UNIVERSE,
};
use super::structured_leaf_candidates::StructuredLeafCandidatesReport;

const DIAGNOSTIC_SCHEMA: &str = "requirement-binding-diagnostic-v1";

#[derive(Serialize)]
struct DiagnosticEnvelope {
    schema_version: &'static str,
    diagnostic_only: bool,
    execution_performed: bool,
    passing_claimed: bool,
    fixed_katana_revision: &'static str,
    source_closure: SourceClosureArtifact,
    branch_catalog: BranchCatalogArtifact,
    action_origins: super::model::ActionOriginsArtifact,
    requirement_bindings: RequirementBindingResult,
    requirement_source_inventory: RequirementSourceInventoryReport,
    all_requirement_bindings: RequirementBindingResult,
    source_action_bindings: SourceActionBindingReport,
    source_leaf_candidates: StructuredLeafCandidatesReport,
}

pub(super) fn audit_requirement_bindings(options: &CliOptions) -> Result<(), String> {
    options.reject_unknown(&["katana-repo", "output"])?;
    let katana_repo = options.one("katana-repo")?;
    let output_arg = options.one("output")?;
    let katana_root = super::operational_paths::canonical_dir(katana_repo, "KatanA")?;
    let output = Path::new(output_arg);
    verify_checkout(&katana_root)?;
    validate_output_location(output, &katana_root)?;
    let docs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs");
    let universe = docs.join("v0-1-0-katana-editor-source-universe.md");
    let roots = docs.join("v0-1-0-source-closure-roots.json");
    let requirements = docs.join("v0-1-0-editor-requirements.md");
    let aliases = docs.join("v0-1-0-editor-requirement-source-aliases.json");
    let universe_bytes = std::fs::read(&universe)
        .map_err(|error| format!("failed to read checked-in source universe: {error}"))?;
    verify_checked_in_input(
        "source universe",
        &universe_bytes,
        SOURCE_UNIVERSE.as_bytes(),
    )?;
    let root_bytes = std::fs::read(&roots)
        .map_err(|error| format!("failed to read checked-in source roots: {error}"))?;
    verify_checked_in_input("source roots", &root_bytes, ROOT_MANIFEST.as_bytes())?;
    let requirements_bytes = std::fs::read(&requirements)
        .map_err(|error| format!("failed to read checked-in requirements: {error}"))?;
    verify_checked_in_input("requirements", &requirements_bytes, REQUIREMENTS.as_bytes())?;
    let aliases_bytes = std::fs::read(&aliases)
        .map_err(|error| format!("failed to read checked-in requirement aliases: {error}"))?;
    verify_checked_in_input(
        "requirement source aliases",
        &aliases_bytes,
        ALIAS_LEDGER.as_bytes(),
    )?;
    let universe_sha = sha256_hex(&universe_bytes);
    let paths = super::source_roots::resolve(&roots, &universe, &katana_root, &universe_sha)?;
    let mut state = ScanState::default();
    let mut queue = paths
        .into_iter()
        .map(|path| katana_root.join(path))
        .collect::<VecDeque<_>>();
    let mut seen = BTreeSet::new();
    let mut invocation_sources = Vec::new();
    while let Some(path) = queue.pop_front() {
        if let Some(source) = scan_source(&katana_root, &path, &mut state, &mut queue, &mut seen)? {
            invocation_sources.push(source);
        }
    }
    state.finalize_unscanned_targets();
    let lock_bytes = read_and_verify(&katana_root, "Cargo.lock")?;
    let external_ui = super::external_ui_dependency::audit_locked_sources(
        &katana_root,
        &lock_bytes,
        &invocation_sources,
    );
    let root = diagnostic_root(
        &state,
        &universe_sha,
        &sha256_hex(&aliases_bytes),
        &external_ui.fingerprint,
    )?;
    let edge_index = state
        .edges
        .iter()
        .map(|edge| (edge.id.clone(), edge.clone()))
        .collect::<BTreeMap<_, _>>();
    let files = state
        .files
        .iter()
        .map(|(path, file)| SourceClosureFile {
            path: path.clone(),
            sha256: file.sha256.clone(),
            incoming_edges: file
                .incoming_edges
                .iter()
                .filter_map(|id| edge_index.get(id).cloned())
                .collect(),
            classification: file.classification.clone(),
            classification_rationale: file.classification_rationale.clone(),
        })
        .collect();
    let source = SourceClosureArtifact {
        root: root.clone(),
        profiles: Vec::new(),
        files,
        external_ui_semantic_dependencies: vec![external_ui],
        unresolved_edges: state.unresolved_edges.clone(),
        source_derived_native_target: None,
        context_menu_target_manifest: None,
    };
    let branches = materialize_branch_catalog(root.clone(), &state, &[], &katana_root)?;
    let actions = materialize_action_origins(root, &state);
    let bindings = RequirementSourceAliasLedger::bind_verified_requirements(&source, &branches)?;
    let inventory = RequirementSourceInventoryReport::build(
        &requirements_bytes,
        &source,
        &RequirementSourceAliasLedger::verified_alias_entries()?,
    )?;
    let all_bindings = RequirementBindingResult::build(&inventory.entries(), &source, &branches)?;
    let source_action_bindings = SourceActionBindingReport::build(&state, &source, &all_bindings)?;
    let source_leaf_candidates = StructuredLeafCandidatesReport::build(
        &source,
        &branches,
        &all_bindings,
        &source_action_bindings,
    )?;
    for (path, file) in &state.files {
        verify_fixed_blob(&katana_root, path, Some(&file.sha256))?;
    }
    verify_fixed_blob(&katana_root, "Cargo.lock", Some(&sha256_hex(&lock_bytes)))?;
    verify_checkout(&katana_root)?;
    let envelope = DiagnosticEnvelope {
        schema_version: DIAGNOSTIC_SCHEMA,
        diagnostic_only: true,
        execution_performed: false,
        passing_claimed: false,
        fixed_katana_revision: FIXED_KATANA_REVISION,
        source_closure: source,
        branch_catalog: branches,
        action_origins: actions,
        requirement_bindings: bindings,
        requirement_source_inventory: inventory,
        all_requirement_bindings: all_bindings,
        source_action_bindings,
        source_leaf_candidates,
    };
    write_new_json(output, &envelope)
}

fn verify_checked_in_input(label: &str, actual: &[u8], expected: &[u8]) -> Result<(), String> {
    if actual != expected {
        return Err(format!(
            "checked-in {label} differs from compiled diagnostic input"
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "requirement_diagnostic_tests.rs"]
mod tests;
