#[test]
fn parsed_rust_fixture_produces_real_binding_facts_and_unbound_branch()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = FixtureBuilder::root().map_err(std::io::Error::other)?;
    let fixture_root = fixture.path().canonicalize()?;
    let source_root = fixture_root.join("crates/katana-ui/src");
    fs::create_dir_all(&source_root)?;
    let lib_source = "mod extra;\nfn run(value: bool) { if value { } }\n";
    let extra_source = "fn extra(value: bool) { if value { } }\n";
    let lib_path = source_root.join("lib.rs");
    let extra_path = source_root.join("extra.rs");
    fs::write(&lib_path, lib_source)?;
    fs::write(&extra_path, extra_source)?;
    let fixture_root = fixture_root.canonicalize()?;

    let mut state = ScanState::default();
    let mut pending = VecDeque::from([fixture_root.join("crates/katana-ui/src/lib.rs")]);
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
        let contents = fs::read_to_string(&path)?;
        let parsed = syn::parse_file(&contents)?;
        let mut discovered = Vec::new();
        SourceClosureVisitor::new(&fixture_root, &path, &relative, &mut state, &mut discovered)
            .scan_file(&parsed);
        pending.extend(discovered);
    }

    let manifest_root = root("revision");
    let catalog = crate::source_closure::branch_catalog::materialize_branch_catalog(
        manifest_root.clone(),
        &state,
        &[],
        &fixture_root,
    )
    .map_err(std::io::Error::other)?;
    let source_artifact = source(
        &manifest_root,
        &[
            (
                "crates/katana-ui/src/lib.rs",
                &sha256_hex(lib_source.as_bytes()),
            ),
            (
                "crates/katana-ui/src/extra.rs",
                &sha256_hex(extra_source.as_bytes()),
            ),
        ],
    );
    let result = RequirementBindingResult::build(
        &[entry("editor.one", "crates/katana-ui/src/lib.rs")],
        &source_artifact,
        &catalog,
    )?;
    let binding = result
        .bindings
        .first()
        .ok_or("parsed fixture did not produce a binding")?;
    let parsed_branch = catalog
        .branches
        .iter()
        .find(|branch| branch.file == "crates/katana-ui/src/lib.rs")
        .ok_or("parsed fixture branch missing")?;
    assert_eq!(binding.branch_id, parsed_branch.branch_id);
    assert!(binding.branch_id.starts_with("branch:sha256:"));
    assert_eq!(binding.span_start_line, parsed_branch.span.start_line);
    assert_eq!(binding.span_end_line, parsed_branch.span.end_line);
    assert_eq!(binding.source_sha256, sha256_hex(lib_source.as_bytes()));
    assert_eq!(
        binding.source_excerpt_sha256,
        parsed_branch.source_excerpt_sha256
    );
    let extra_branch = catalog
        .branches
        .iter()
        .find(|branch| branch.file == "crates/katana-ui/src/extra.rs")
        .ok_or("parsed fixture extra branch missing")?;
    assert!(
        result
            .unbound_branches
            .iter()
            .any(|branch| branch.branch_id == extra_branch.branch_id)
    );
    Ok(())
}

#[test]
fn duplicate_unbound_branch_ids_fail_before_reverse_join() -> Result<(), Box<dyn std::error::Error>>
{
    let manifest_root = root("revision");
    let source_artifact = source(&manifest_root, &[]);
    let mut catalog = branch(
        &manifest_root,
        "crates/katana-ui/src/other.rs",
        "branch:one",
    );
    catalog.branches[0].file = "crates/katana-ui/src/other.rs".into();
    catalog.branches.push(catalog.branches[0].clone());
    assert!(matches!(
        RequirementBindingResult::build(&[], &source_artifact, &catalog),
        Err(error) if error.contains("duplicate branch id")
    ));
    Ok(())
}

#[test]
fn source_and_branch_changes_change_fingerprint() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_root = root("revision");
    let entries = [entry("editor.one", "crates/katana-ui/src/editor.rs")];
    let first_source = source(
        &manifest_root,
        &[("crates/katana-ui/src/editor.rs", "sha256:file")],
    );
    let first_branch = branch(
        &manifest_root,
        "crates/katana-ui/src/editor.rs",
        "branch:one",
    );
    let first =
        RequirementBindingResult::build(&entries, &first_source, &first_branch)?.fingerprint;
    let changed_source = source(
        &manifest_root,
        &[("crates/katana-ui/src/editor.rs", "sha256:changed")],
    );
    let changed =
        RequirementBindingResult::build(&entries, &changed_source, &first_branch)?.fingerprint;
    assert_ne!(first, changed);
    let changed_branch = branch(
        &manifest_root,
        "crates/katana-ui/src/editor.rs",
        "branch:changed",
    );
    let changed =
        RequirementBindingResult::build(&entries, &first_source, &changed_branch)?.fingerprint;
    assert_ne!(first, changed);

    let mut changed_span_start = first_branch.clone();
    changed_span_start.branches[0].span.start_line += 1;
    let changed =
        RequirementBindingResult::build(&entries, &first_source, &changed_span_start)?.fingerprint;
    assert_ne!(first, changed);

    let mut changed_span_end = first_branch.clone();
    changed_span_end.branches[0].span.end_line += 1;
    let changed =
        RequirementBindingResult::build(&entries, &first_source, &changed_span_end)?.fingerprint;
    assert_ne!(first, changed);

    let mut changed_excerpt = first_branch.clone();
    changed_excerpt.branches[0].source_excerpt_sha256 = "sha256:changed-excerpt".into();
    let changed =
        RequirementBindingResult::build(&entries, &first_source, &changed_excerpt)?.fingerprint;
    assert_ne!(first, changed);

    let mut later_root = manifest_root.clone();
    later_root.generated_at_utc = "2026-09-05T00:00:01Z".into();
    let later_source = source(
        &later_root,
        &[("crates/katana-ui/src/editor.rs", "sha256:file")],
    );
    let later_branch = branch(&later_root, "crates/katana-ui/src/editor.rs", "branch:one");
    let later =
        RequirementBindingResult::build(&entries, &later_source, &later_branch)?.fingerprint;
    assert_ne!(first, later);
    Ok(())
}

#[test]
fn input_order_and_json_bytes_are_deterministic() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_root = root("revision");
    let entries = [
        entry("editor.two", "crates/katana-ui/src/editor.rs"),
        entry("editor.one", "crates/katana-ui/src/editor.rs"),
    ];
    let source_artifact = source(
        &manifest_root,
        &[("crates/katana-ui/src/editor.rs", "sha256:file")],
    );
    let mut catalog = branch(
        &manifest_root,
        "crates/katana-ui/src/editor.rs",
        "branch:one",
    );
    catalog.branches.push(BranchRecord {
        branch_id: "branch:two".into(),
        ..catalog.branches[0].clone()
    });
    let first = RequirementBindingResult::build(&entries, &source_artifact, &catalog)?;
    let reversed_entries = [entries[1].clone(), entries[0].clone()];
    let mut reversed_catalog = catalog.clone();
    reversed_catalog.branches.reverse();
    let second =
        RequirementBindingResult::build(&reversed_entries, &source_artifact, &reversed_catalog)?;
    assert_eq!(first.fingerprint, second.fingerprint);
    assert_eq!(serde_json::to_vec(&first)?, serde_json::to_vec(&second)?);
    Ok(())
}

#[test]
fn unbound_branch_facts_change_fingerprint() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_root = root("revision");
    let source_artifact = source(&manifest_root, &[]);
    let mut base = branch(
        &manifest_root,
        "crates/katana-ui/src/other.rs",
        "branch:one",
    );
    base.branches[0].symbol = "run".into();
    base.branches[0].kind = "cfg".into();
    base.branches[0].condition = "unix".into();
    base.branches[0].active_profile_ids = vec!["macos".into()];
    base.branches[0].inactive_profile_predicates = vec![InactiveProfilePredicate {
        profile_id: "windows".into(),
        predicate: "target_os = windows".into(),
    }];
    base.branches[0].incoming_edges = vec!["edge:one".into()];
    let first = RequirementBindingResult::build(&[], &source_artifact, &base)?.fingerprint;
    for mutate in [
        change_unbound_symbol as fn(&mut BranchRecord),
        change_unbound_kind,
        change_unbound_condition,
        change_unbound_profiles,
        change_unbound_predicates,
        change_unbound_edges,
    ] {
        let mut changed = base.clone();
        mutate(&mut changed.branches[0]);
        let fingerprint =
            RequirementBindingResult::build(&[], &source_artifact, &changed)?.fingerprint;
        assert_ne!(first, fingerprint);
    }
    Ok(())
}

fn change_unbound_symbol(branch: &mut BranchRecord) {
    branch.symbol.push_str("-changed");
}

fn change_unbound_kind(branch: &mut BranchRecord) {
    branch.kind.push_str("-changed");
}

fn change_unbound_condition(branch: &mut BranchRecord) {
    branch.condition.push_str("-changed");
}

fn change_unbound_profiles(branch: &mut BranchRecord) {
    branch.active_profile_ids.push("linux".into());
}

fn change_unbound_predicates(branch: &mut BranchRecord) {
    branch.inactive_profile_predicates[0]
        .predicate
        .push_str("-changed");
}

fn change_unbound_edges(branch: &mut BranchRecord) {
    branch.incoming_edges.push("edge:changed".into());
}
use std::collections::{BTreeSet, VecDeque};
use std::fs;

use super::super::super::artifact_model::{BranchRecord, InactiveProfilePredicate};
use super::super::super::ast_scan::SourceClosureVisitor;
use super::super::super::fingerprint::sha256_hex;
use super::super::super::requirement_binding::RequirementBindingResult;
use super::super::super::scan_state::ScanState;
use super::support::{branch, entry, root, source};
use crate::capability_manifest::capability_manifest_test_fixtures::FixtureBuilder;
