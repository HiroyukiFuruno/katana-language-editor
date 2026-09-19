use std::collections::{BTreeSet, VecDeque};
use std::fs;

use crate::capability_manifest::capability_manifest_test_fixtures::FixtureBuilder;
use crate::source_closure::artifact_model::SourceClosureArtifact;
use crate::source_closure::ast_scan::SourceClosureVisitor;
use crate::source_closure::branch_catalog::materialize_branch_catalog;
use crate::source_closure::fingerprint::sha256_hex;
use crate::source_closure::model::{ManifestRoot, SourceClosureFile};
use crate::source_closure::requirement_binding::RequirementBindingResult;
use crate::source_closure::scan_state::ScanState;
use crate::source_closure::source_requirement_alias_ledger::AliasEntry;

fn root(revision: &str) -> ManifestRoot {
    ManifestRoot {
        schema_version: "requirement-binding-diagnostic-v1".into(),
        katana_revision: revision.into(),
        katana_tree_fingerprint: "tree".into(),
        katana_external_ui_fingerprint: "external".into(),
        user_mandated_extensions_fingerprint: "extensions".into(),
        source_universe_fingerprint: "universe".into(),
        requirement_source_aliases_fingerprint: "aliases".into(),
        kle_tree_fingerprint: "kle".into(),
        kuc_tree_fingerprint: "kuc".into(),
        release_profile_matrix_fingerprint: "profiles".into(),
        generator_fingerprint: "generator".into(),
        generated_at_utc: "2026-09-05T00:00:00Z".into(),
        static_leaf_count: None,
        expected_leaf_ids: None,
    }
}

pub(in crate::source_closure) struct FixtureFacts {
    pub(in crate::source_closure) _fixture:
        crate::capability_manifest::capability_manifest_test_fixtures::FixtureDirectory,
    pub(in crate::source_closure) root: ManifestRoot,
    pub(in crate::source_closure) state: ScanState,
    pub(in crate::source_closure) source: SourceClosureArtifact,
    pub(in crate::source_closure) requirements: RequirementBindingResult,
}

pub(in crate::source_closure) fn actual_fixture() -> Result<FixtureFacts, Box<dyn std::error::Error>>
{
    let fixture = FixtureBuilder::root().map_err(std::io::Error::other)?;
    let fixture_root = fixture.path().canonicalize()?;
    let source_root = fixture_root.join("crates/katana-ui/src");
    fs::create_dir_all(&source_root)?;
    let lib_source = "mod app_action;\nmod editor;\nmod dispatch;\n";
    let definition_source = "enum AppAction { Bold, Other, Unrelated }\n";
    let construction_source = "fn emit(value: bool) {\n    if value {\n        let _inside = AppAction::Bold;\n    }\n    let _outside = AppAction::Bold;\n    let _unrelated = AppAction::Other;\n}\n";
    let dispatch_source = "fn handle_bold() {}\nfn handle_other() {}\nfn dispatch_action(action: AppAction) {\n    match action {\n        AppAction::Bold => handle_bold(),\n        AppAction::Other => handle_other(),\n    }\n}\n";
    let source_path = source_root.join("lib.rs");
    fs::write(&source_path, lib_source)?;
    fs::write(source_root.join("app_action.rs"), definition_source)?;
    fs::write(source_root.join("editor.rs"), construction_source)?;
    fs::write(source_root.join("dispatch.rs"), dispatch_source)?;

    let mut state = ScanState::default();
    let mut pending = VecDeque::from([source_path]);
    let mut seen = BTreeSet::new();
    while let Some(path) = pending.pop_front() {
        let path = path.canonicalize()?;
        if !seen.insert(path.clone()) {
            continue;
        }
        let relative = path
            .strip_prefix(&fixture_root)?
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/");
        let bytes = fs::read(&path)?;
        let source = std::str::from_utf8(&bytes)?;
        let parsed = syn::parse_file(source)?;
        state.record_file(relative.clone(), sha256_hex(&bytes));
        let mut discovered = Vec::new();
        SourceClosureVisitor::new(&fixture_root, &path, &relative, &mut state, &mut discovered)
            .scan_file(&parsed);
        pending.extend(discovered);
    }

    let manifest_root = root("revision");
    let branches = materialize_branch_catalog(manifest_root.clone(), &state, &[], &fixture_root)
        .map_err(std::io::Error::other)?;
    let files = state
        .files
        .iter()
        .map(|(path, file)| SourceClosureFile {
            path: path.clone(),
            sha256: file.sha256.clone(),
            incoming_edges: Vec::new(),
            classification: file.classification.clone(),
            classification_rationale: file.classification_rationale.clone(),
        })
        .collect();
    let source = SourceClosureArtifact {
        root: manifest_root.clone(),
        profiles: Vec::new(),
        files,
        external_ui_semantic_dependencies: Vec::new(),
        unresolved_edges: Vec::new(),
        source_derived_native_target: None,
        context_menu_target_manifest: None,
    };
    let requirements = RequirementBindingResult::build(
        &[AliasEntry {
            requirement_id: "editor.one".into(),
            short_reference: "lib.rs".into(),
            source_path: "crates/katana-ui/src/editor.rs".into(),
        }],
        &source,
        &branches,
    )
    .map_err(std::io::Error::other)?;
    Ok(FixtureFacts {
        _fixture: fixture,
        root: manifest_root,
        state,
        source,
        requirements,
    })
}
