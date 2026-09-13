use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::super::ActualReferenceVerifier;
use super::super::SourceClosureMaterializer;
use super::super::fingerprint::sha256_hex;
use super::super::operational_input::FIXED_KATANA_REVISION;
use super::super::operational_loader::SourceClosureInputLoaderVerifier;
use super::operational_fixture::{ACTUAL_KATANA_ROOT, DEFAULT_TREE_PATHS, fixture};
use super::operational_fixture_root::fixed_reference_root;
use super::operational_test_support::{
    TestResult, cleanup_dir, mutate_input, option_result, reject, string_result, test_error,
};

mod action_candidate_tests;
mod branch_catalog_tests;
mod branch_effect_tests;
mod closure_tests;
mod fixture_reference_tests;
mod handler_continuation_tests;
mod handler_definition_tests;
mod handler_tests;
mod input_candidate_tests;
mod profile_tests;
mod publication_tests;

fn action_origins_bytes_for_source(name: &str, source: &str) -> TestResult<Vec<u8>> {
    let (state, _discovered, root) =
        scan_source_fixture(name, &[("src/lib.rs", source)], "src/lib.rs")?;
    let artifact =
        super::super::action_origins::materialize_action_origins(test_manifest_root(), &state);
    let mut bytes = serde_json::to_vec_pretty(&artifact)?;
    bytes.push(b'\n');
    cleanup_dir(root)?;
    Ok(bytes)
}

fn scan_source_fixture(
    name: &str,
    files: &[(&str, &str)],
    entry: &str,
) -> TestResult<(super::super::scan_state::ScanState, Vec<PathBuf>, PathBuf)> {
    let root = std::env::temp_dir().join(format!(
        "kpc-materializer-{name}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| test_error(format!("fixture clock failed: {error}")))?
            .as_nanos()
    ));
    for (relative, source) in files {
        let path = root.join(relative);
        let parent = option_result(
            path.parent(),
            format!("fixture path has no parent: {}", path.display()),
        )?;
        fs::create_dir_all(parent)?;
        fs::write(path, source)?;
    }
    let root = root.canonicalize()?;

    let mut state = super::super::scan_state::ScanState::default();
    let mut discovered = Vec::new();
    let mut pending = vec![root.join(entry)];
    let mut seen = BTreeSet::new();
    while let Some(source_path) = pending.pop() {
        let source_path = source_path.canonicalize()?;
        if !seen.insert(source_path.clone()) {
            continue;
        }
        let relative = source_path
            .strip_prefix(&root)
            .map_err(|error| test_error(format!("fixture relative path failed: {error}")))?
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/");
        let source = fs::read_to_string(&source_path)?;
        let file = syn::parse_file(&source)?;
        let before = discovered.len();
        super::super::ast_scan::SourceClosureVisitor::new(
            &root,
            &source_path,
            &relative,
            &mut state,
            &mut discovered,
        )
        .scan_file(&file);
        pending.extend(discovered[before..].iter().cloned());
    }
    Ok((state, discovered, root))
}

fn catalog_bytes_for_source(name: &str, source: &str) -> TestResult<Vec<u8>> {
    let catalog = catalog_for_source(name, source)?;
    let mut bytes = serde_json::to_vec_pretty(&catalog)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn catalog_for_source(
    name: &str,
    source: &str,
) -> TestResult<super::super::model::BranchCatalogArtifact> {
    let (state, _discovered, root) =
        scan_source_fixture(name, &[("src/lib.rs", source)], "src/lib.rs")?;
    let catalog = string_result(super::super::branch_catalog::materialize_branch_catalog(
        test_manifest_root(),
        &state,
        &test_profiles(),
        &root,
    ))?;
    cleanup_dir(root)?;
    Ok(catalog)
}

fn test_profiles() -> Vec<super::super::model::ProfileRecord> {
    super::super::model::CANONICAL_PROFILE_IDS
        .iter()
        .map(|id| super::super::model::ProfileRecord {
            id: (*id).to_string(),
            runner_label: (*id).to_string(),
            katana_revision: super::super::operational_input::FIXED_KATANA_REVISION.to_string(),
            fingerprint: format!("{id}-fingerprint"),
            rustc_host_triple: format!("{id}-host"),
            rustc_cfg_sha256: format!("{id}-cfg"),
            cargo_resolution_sha256: format!("{id}-cargo"),
            lockfile_sha256: format!("{id}-lock"),
            active_edge_ids: vec![],
            inactive_cfg_edges: vec![],
        })
        .collect()
}

fn test_manifest_root() -> super::super::model::ManifestRoot {
    super::super::model::ManifestRoot {
        schema_version: "source-closure.v1".to_string(),
        katana_revision: "revision".to_string(),
        katana_tree_fingerprint: "tree".to_string(),
        katana_external_ui_fingerprint: "external".to_string(),
        user_mandated_extensions_fingerprint: "extensions".to_string(),
        source_universe_fingerprint: "source-universe".to_string(),
        requirement_source_aliases_fingerprint: "aliases".to_string(),
        kle_tree_fingerprint: "kle".to_string(),
        kuc_tree_fingerprint: "kuc".to_string(),
        release_profile_matrix_fingerprint: "profiles".to_string(),
        generator_fingerprint: "generator".to_string(),
        generated_at_utc: "2026-08-21T00:00:00Z".to_string(),
        static_leaf_count: None,
        expected_leaf_ids: None,
    }
}
