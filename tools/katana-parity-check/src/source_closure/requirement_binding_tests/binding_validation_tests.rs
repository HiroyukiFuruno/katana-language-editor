#[test]
fn binds_sha_span_root_and_keeps_unbound_branches() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_root = root("revision");
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
        branch_id: "branch:unbound".into(),
        file: "crates/katana-ui/src/other.rs".into(),
        symbol: "other".into(),
        span: TextSpan {
            start_line: 4,
            end_line: 4,
        },
        kind: "match".into(),
        condition: "Some(_)".into(),
        active_profile_ids: vec!["macos".into(), "linux".into()],
        inactive_profile_predicates: vec![InactiveProfilePredicate {
            profile_id: "windows".into(),
            predicate: "target_os = windows".into(),
        }],
        outcomes: Vec::new(),
        incoming_edges: vec!["edge:incoming".into()],
        source_excerpt_sha256: "sha256:other".into(),
    });
    let result = RequirementBindingResult::build(
        &[entry("editor.one", "crates/katana-ui/src/editor.rs")],
        &source_artifact,
        &catalog,
    )?;
    assert_eq!(result.bindings.len(), 1);
    assert_eq!(result.bindings[0].source_sha256, "sha256:file");
    assert_eq!(result.bindings[0].span_start_line, 2);
    assert_eq!(result.bindings[0].source_excerpt_sha256, "sha256:excerpt");
    assert_eq!(result.unbound_branches.len(), 1);
    assert_eq!(result.unbound_branches[0].branch_id, "branch:unbound");
    assert_eq!(result.unbound_branches[0].symbol, "other");
    assert_eq!(result.unbound_branches[0].kind, "match");
    assert_eq!(result.unbound_branches[0].condition, "Some(_)");
    assert_eq!(
        result.unbound_branches[0].active_profile_ids,
        ["macos", "linux"]
    );
    assert_eq!(
        serde_json::to_value(&result.unbound_branches[0].inactive_profile_predicates)?,
        serde_json::json!([{"profile_id": "windows", "predicate": "target_os = windows"}])
    );
    assert_eq!(result.unbound_branches[0].incoming_edges, ["edge:incoming"]);
    assert_eq!(
        serde_json::to_value(&result.root)?,
        serde_json::to_value(&manifest_root)?
    );
    let _: RequirementBindingResult = result;
    Ok(())
}

#[test]
fn missing_duplicate_and_branchless_files_are_unresolved() -> Result<(), Box<dyn std::error::Error>>
{
    let manifest_root = root("revision");
    let source_artifact = source(
        &manifest_root,
        &[
            ("crates/katana-ui/src/duplicate.rs", "sha256:a"),
            ("crates/katana-ui/src/duplicate.rs", "sha256:b"),
            ("crates/katana-ui/src/no_branch.rs", "sha256:c"),
        ],
    );
    let catalog = branch(
        &manifest_root,
        "crates/katana-ui/src/other.rs",
        "branch:other",
    );
    let entries = vec![
        entry("missing", "crates/katana-ui/src/missing.rs"),
        entry("duplicate", "crates/katana-ui/src/duplicate.rs"),
        entry("branchless", "crates/katana-ui/src/no_branch.rs"),
    ];
    let result = RequirementBindingResult::build(&entries, &source_artifact, &catalog)?;
    assert!(result.bindings.is_empty());
    assert_eq!(result.unresolved.len(), 3);
    assert!(
        result
            .unresolved
            .iter()
            .any(|item| item.reason == "source file not found")
    );
    assert!(
        result
            .unresolved
            .iter()
            .any(|item| item.reason.contains("duplicated"))
    );
    assert!(
        result
            .unresolved
            .iter()
            .any(|item| item.reason == "no branch found in source file")
    );
    assert_eq!(result.unbound_branches.len(), 1);
    Ok(())
}

#[test]
fn duplicate_source_key_fails_closed() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_root = root("revision");
    let source_artifact = source(
        &manifest_root,
        &[("crates/katana-ui/src/editor.rs", "sha256:file")],
    );
    let catalog = branch(
        &manifest_root,
        "crates/katana-ui/src/editor.rs",
        "branch:one",
    );
    let duplicate = vec![
        entry("editor.one", "crates/katana-ui/src/editor.rs"),
        entry("editor.one", "crates/katana-ui/src/editor.rs"),
    ];
    assert!(matches!(
        RequirementBindingResult::build(&duplicate, &source_artifact, &catalog),
        Err(error) if error.contains("duplicate requirement source key")
    ));
    Ok(())
}

#[test]
fn every_manifest_root_field_must_match() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_root = root("revision");
    let source_artifact = source(
        &manifest_root,
        &[("crates/katana-ui/src/editor.rs", "sha256:file")],
    );
    let mutations: [RootMutation; 14] = [
        ("schema_version", |value| {
            value.schema_version.push_str("-other")
        }),
        ("katana_revision", |value| {
            value.katana_revision.push_str("-other")
        }),
        ("katana_tree_fingerprint", |value| {
            value.katana_tree_fingerprint.push_str("-other")
        }),
        ("katana_external_ui_fingerprint", |value| {
            value.katana_external_ui_fingerprint.push_str("-other")
        }),
        ("user_mandated_extensions_fingerprint", |value| {
            value
                .user_mandated_extensions_fingerprint
                .push_str("-other")
        }),
        ("source_universe_fingerprint", |value| {
            value.source_universe_fingerprint.push_str("-other")
        }),
        ("requirement_source_aliases_fingerprint", |value| {
            value
                .requirement_source_aliases_fingerprint
                .push_str("-other")
        }),
        ("kle_tree_fingerprint", |value| {
            value.kle_tree_fingerprint.push_str("-other")
        }),
        ("kuc_tree_fingerprint", |value| {
            value.kuc_tree_fingerprint.push_str("-other")
        }),
        ("release_profile_matrix_fingerprint", |value| {
            value.release_profile_matrix_fingerprint.push_str("-other")
        }),
        ("generator_fingerprint", |value| {
            value.generator_fingerprint.push_str("-other")
        }),
        ("generated_at_utc", |value| {
            value.generated_at_utc.push_str("-other")
        }),
        ("static_leaf_count", |value| {
            value.static_leaf_count = Some(1)
        }),
        ("expected_leaf_ids", |value| {
            value.expected_leaf_ids = Some(vec!["leaf".into()])
        }),
    ];
    for (field, mutate) in mutations {
        let mut mismatched_root = manifest_root.clone();
        mutate(&mut mismatched_root);
        let mismatched_catalog = branch(
            &mismatched_root,
            "crates/katana-ui/src/editor.rs",
            "branch:one",
        );
        assert!(
            matches!(
                RequirementBindingResult::build(
                    &[entry("editor.one", "crates/katana-ui/src/editor.rs")],
                    &source_artifact,
                    &mismatched_catalog,
                ),
                Err(error) if error.contains("roots differ")
            ),
            "root field {field} was not checked"
        );
    }
    Ok(())
}

#[test]
fn verified_wrapper_rejects_same_foreign_root_before_alias_join()
-> Result<(), Box<dyn std::error::Error>> {
    let foreign_root = root("foreign-revision");
    let source_artifact = source(&foreign_root, &[]);
    let branches = BranchCatalogArtifact {
        root: foreign_root,
        branches: Vec::new(),
        unclassified_branch_ids: Vec::new(),
    };
    assert!(matches!(
        RequirementSourceAliasLedger::bind_verified_requirements(&source_artifact, &branches),
        Err(error) if error.contains("foreign KatanA revision")
    ));
    Ok(())
}

#[test]
fn verified_wrapper_requires_checked_in_alias_and_universe_fingerprints()
-> Result<(), Box<dyn std::error::Error>> {
    for (field, error_fragment) in [
        ("alias", "foreign alias ledger fingerprint"),
        ("universe", "foreign source universe fingerprint"),
    ] {
        let mut mismatched_root = root(FIXED_KATANA_REVISION);
        mismatched_root.requirement_source_aliases_fingerprint =
            sha256_hex(ALIAS_LEDGER.as_bytes());
        mismatched_root.source_universe_fingerprint = sha256_hex(SOURCE_UNIVERSE.as_bytes());
        if field == "alias" {
            mismatched_root.requirement_source_aliases_fingerprint = "foreign-alias".into();
        } else {
            mismatched_root.source_universe_fingerprint = "foreign-universe".into();
        }
        let source_artifact = source(&mismatched_root, &[]);
        let branches = BranchCatalogArtifact {
            root: mismatched_root,
            branches: Vec::new(),
            unclassified_branch_ids: Vec::new(),
        };
        assert!(matches!(
            RequirementSourceAliasLedger::bind_verified_requirements(&source_artifact, &branches),
            Err(error) if error.contains(error_fragment)
        ));
    }
    assert!(!ALIAS_LEDGER.is_empty());
    assert!(!SOURCE_UNIVERSE.is_empty());
    Ok(())
}
use super::super::super::artifact_model::{
    BranchCatalogArtifact, BranchRecord, InactiveProfilePredicate,
};
use super::super::super::model::TextSpan;
use super::super::super::operational_input::FIXED_KATANA_REVISION;
use super::super::super::requirement_binding::RequirementBindingResult;
use super::super::super::source_requirement_alias_ledger::{
    ALIAS_LEDGER, RequirementSourceAliasLedger, SOURCE_UNIVERSE,
};
use super::support::{RootMutation, branch, entry, root, source};
use crate::source_closure::fingerprint::sha256_hex;
