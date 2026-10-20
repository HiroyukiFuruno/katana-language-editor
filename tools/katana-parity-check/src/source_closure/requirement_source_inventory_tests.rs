use super::artifact_model::SourceClosureArtifact;
use super::edge_model::SourceClosureFile;
use super::requirement_source_inventory::RequirementSourceInventoryReport;
use super::root_model::ManifestRoot;
use super::source_requirement_alias_ledger::AliasEntry;

const REQUIREMENTS: &str = "## Editor Parity Requirements\n\
| `one` | C/M | `crates/katana-ui/src/views/app_frame/{a,b}.rs`, views/app_frame/** | visible |\n\
| `two` | C/M | external.rs | visible |\n\
### Non-Aggregate Authoring Leaf Catalog\n";

fn source(paths: &[&str]) -> SourceClosureArtifact {
    SourceClosureArtifact {
        root: ManifestRoot {
            schema_version: "1".to_string(),
            katana_revision: "rev".to_string(),
            katana_tree_fingerprint: "tree".to_string(),
            katana_external_ui_fingerprint: "external".to_string(),
            user_mandated_extensions_fingerprint: "extensions".to_string(),
            source_universe_fingerprint: "universe".to_string(),
            requirement_source_aliases_fingerprint: "aliases".to_string(),
            kle_tree_fingerprint: "kle".to_string(),
            kuc_tree_fingerprint: "kuc".to_string(),
            release_profile_matrix_fingerprint: "matrix".to_string(),
            generator_fingerprint: "generator".to_string(),
            generated_at_utc: "now".to_string(),
            static_leaf_count: None,
            expected_leaf_ids: None,
        },
        profiles: vec![],
        files: paths
            .iter()
            .map(|path| SourceClosureFile {
                path: (*path).to_string(),
                sha256: super::fingerprint::sha256_hex(fixture_bytes(path)),
                incoming_edges: vec![],
                classification: "source".to_string(),
                classification_rationale: "fixture".to_string(),
            })
            .collect(),
        external_ui_semantic_dependencies: vec![],
        unresolved_edges: vec![],
        source_derived_native_target: None,
        context_menu_target_manifest: None,
    }
}

fn fixture_bytes(path: &str) -> &'static [u8] {
    let bytes: &'static [u8] = match path.rsplit('/').next() {
        Some("a.rs") => b"fn a() { let _value = 1; }",
        Some("b.rs") => b"fn b() { let _value = 2; }",
        Some("c.rs") => b"fn c() { let _value = 3; }",
        _ => b"fn source() {}",
    };
    assert!(
        std::str::from_utf8(bytes)
            .ok()
            .and_then(|text| syn::parse_file(text).ok())
            .is_some()
    );
    bytes
}

#[test]
fn inventory_expands_references_and_merges_alias_provenance() -> Result<(), String> {
    let report = RequirementSourceInventoryReport::build(
        REQUIREMENTS.as_bytes(),
        &source(&[
            "crates/katana-ui/src/views/app_frame/a.rs",
            "crates/katana-ui/src/views/app_frame/b.rs",
            "crates/katana-ui/src/views/app_frame/c.rs",
        ]),
        &[AliasEntry {
            requirement_id: "one".to_string(),
            short_reference: "legacy.rs".to_string(),
            source_path: "crates/katana-ui/src/views/app_frame/a.rs".to_string(),
        }],
    )?;
    assert_eq!(report.entries().len(), 3);
    assert_eq!(report.rows[0].references.len(), 6);
    assert!(
        report.rows[1]
            .unresolved
            .iter()
            .any(|reason| reason.contains("unknown source reference"))
    );
    assert_eq!(report.rows[0].requirement_id, "one");
    let entry = report
        .entries
        .iter()
        .find(|entry| entry.source_path.ends_with("/a.rs"))
        .ok_or("missing a.rs inventory entry")?;
    assert!(entry.provenance.contains(&"legacy.rs".to_string()));
    assert!(
        entry
            .provenance
            .contains(&"crates/katana-ui/src/views/app_frame/a.rs".to_string())
    );
    assert!(
        entry
            .provenance
            .contains(&"crates/katana-ui/src/views/app_frame/**".to_string())
    );
    Ok(())
}

#[test]
fn inventory_is_order_stable_and_rejects_duplicate_ids() -> Result<(), String> {
    let first = RequirementSourceInventoryReport::build(
        REQUIREMENTS.as_bytes(),
        &source(&[
            "crates/katana-ui/src/views/app_frame/c.rs",
            "crates/katana-ui/src/views/app_frame/a.rs",
            "crates/katana-ui/src/views/app_frame/b.rs",
        ]),
        &[],
    )?;
    let second = RequirementSourceInventoryReport::build(
        REQUIREMENTS.as_bytes(),
        &source(&[
            "crates/katana-ui/src/views/app_frame/b.rs",
            "crates/katana-ui/src/views/app_frame/a.rs",
            "crates/katana-ui/src/views/app_frame/c.rs",
        ]),
        &[],
    )?;
    assert_eq!(first.entries(), second.entries());
    assert_eq!(
        serde_json::to_vec(&first).map_err(|error| error.to_string())?,
        serde_json::to_vec(&second).map_err(|error| error.to_string())?,
    );
    let duplicate = REQUIREMENTS.replace("| `two`", "| `one`");
    assert!(
        RequirementSourceInventoryReport::build(duplicate.as_bytes(), &source(&[]), &[],).is_err()
    );
    Ok(())
}

#[path = "requirement_reference_collision_tests.rs"]
mod reference_collision_tests;

#[test]
fn inventory_retains_unscanned_and_unexpanded_rows() -> Result<(), String> {
    let requirements = "## Editor Parity Requirements\n\
| `missing` | C/M | `app_frame/missing.rs`, views/app_frame/empty/** | visible |\n\
### Non-Aggregate Authoring Leaf Catalog\n";
    let report =
        RequirementSourceInventoryReport::build(requirements.as_bytes(), &source(&[]), &[])?;
    let unresolved = &report.rows[0].unresolved;
    assert!(unresolved.iter().any(|reason| reason.contains("unscanned")));
    assert!(
        unresolved
            .iter()
            .any(|reason| reason.contains("unexpanded"))
    );
    Ok(())
}

#[test]
fn empty_and_implicit_sources_remain_unresolved() -> Result<(), String> {
    let requirements = "## Editor Parity Requirements\n\
| `empty` | C/M | | visible |\n\
| `implicit` | C/M | same fixed source | visible |\n\
### Non-Aggregate Authoring Leaf Catalog\n";
    let report =
        RequirementSourceInventoryReport::build(requirements.as_bytes(), &source(&[]), &[])?;
    assert_eq!(report.rows.len(), 2);
    assert!(report.entries.is_empty());
    assert!(
        report.rows[0]
            .unresolved
            .iter()
            .any(|reason| reason.contains("missing"))
    );
    assert!(
        report.rows[1]
            .unresolved
            .iter()
            .any(|reason| reason.contains("same fixed source"))
    );
    let changed = requirements.replace("same fixed source", "other fixed source");
    let changed = RequirementSourceInventoryReport::build(changed.as_bytes(), &source(&[]), &[])?;
    assert_ne!(report.requirements_sha256, changed.requirements_sha256);
    Ok(())
}

#[test]
fn known_alias_does_not_hide_unknown_reference_in_the_same_row() -> Result<(), String> {
    let requirements = "## Editor Parity Requirements\n\
| `one` | C/M | legacy.rs, unknown.rs | visible |\n\
### Non-Aggregate Authoring Leaf Catalog\n";
    let alias = AliasEntry {
        requirement_id: "one".into(),
        short_reference: "legacy.rs".into(),
        source_path: "crates/katana-ui/src/views/app_frame/a.rs".into(),
    };
    let source = source(&["crates/katana-ui/src/views/app_frame/a.rs"]);
    let report = RequirementSourceInventoryReport::build(
        requirements.as_bytes(),
        &source,
        std::slice::from_ref(&alias),
    )?;
    assert_eq!(report.entries.len(), 1);
    assert_eq!(
        report.rows[0].unresolved,
        vec!["unknown source reference: unknown.rs"]
    );
    let mut foreign = alias.clone();
    foreign.requirement_id = "absent".into();
    assert!(
        RequirementSourceInventoryReport::build(requirements.as_bytes(), &source, &[foreign])
            .is_err()
    );
    assert!(
        RequirementSourceInventoryReport::build(
            requirements.as_bytes(),
            &source,
            &[alias.clone(), alias]
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn directory_expansion_rejects_duplicate_sources_and_prefix_lookalikes() -> Result<(), String> {
    let requirements = "## Editor Parity Requirements\n\
| `one` | C/M | views/app_frame/** | visible |\n\
### Non-Aggregate Authoring Leaf Catalog\n";
    let path = "crates/katana-ui/src/views/app_frame/a.rs";
    let report = RequirementSourceInventoryReport::build(
        requirements.as_bytes(),
        &source(&[path, path]),
        &[],
    )?;
    assert!(report.entries.is_empty());
    assert!(
        report.rows[0]
            .unresolved
            .iter()
            .any(|reason| reason.contains("duplicate scanned source"))
    );
    let lookalike = "crates/katana-ui/src/views/app_frame-extra/a.rs";
    let report = RequirementSourceInventoryReport::build(
        requirements.as_bytes(),
        &source(&[lookalike]),
        &[],
    )?;
    assert!(report.entries.is_empty());
    assert!(
        report.rows[0]
            .unresolved
            .iter()
            .any(|reason| reason.contains("unexpanded directory"))
    );
    Ok(())
}

#[test]
fn malformed_requirement_rows_fail_instead_of_disappearing() {
    for row in ["| `` | C/M | app/editor.rs |", "| `missing` |"] {
        let requirements = format!(
            "## Editor Parity Requirements\n{row}\n### Non-Aggregate Authoring Leaf Catalog\n"
        );
        assert!(
            RequirementSourceInventoryReport::build(requirements.as_bytes(), &source(&[]), &[])
                .is_err()
        );
    }
}
