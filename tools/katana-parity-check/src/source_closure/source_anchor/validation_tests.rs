use super::super::model::{AnchorKind, AnchoredFile, SCHEMA, SourceAnchorCatalog};
use super::super::{generation::scan_aliases, scan::scan_file};
use super::expected_from_embedded_source;
use crate::source_closure::{
    artifact_model::SourceClosureArtifact,
    edge_model::SourceClosureFile,
    fingerprint::sha256_hex,
    operational_input::FIXED_KATANA_REVISION,
    root_model::ManifestRoot,
    source_requirement_alias_ledger::{ALIAS_LEDGER, RequirementSourceAliasLedger},
};

const FIXTURE: &str = include_str!("fixtures/anchors.rs");

fn fixture_catalog() -> Result<(SourceClosureArtifact, Vec<u8>, SourceAnchorCatalog), String> {
    let aliases = RequirementSourceAliasLedger::verified_alias_entries()?;
    let source_hash = sha256_hex(FIXTURE.as_bytes());
    let mut by_path = std::collections::BTreeMap::<String, Vec<_>>::new();
    for alias in aliases {
        by_path
            .entry(alias.source_path.clone())
            .or_default()
            .push(alias);
    }
    let syntax = syn::parse_file(FIXTURE).map_err(|error| error.to_string())?;
    let (_, unresolved_syntax) = scan_file(FIXTURE, &syntax)?;
    let mut files = Vec::new();
    let mut closure_files = Vec::new();
    for (path, path_aliases) in by_path {
        let anchors = scan_aliases(FIXTURE, &syntax, &path_aliases)?;
        files.push(AnchoredFile {
            path: path.clone(),
            sha256: source_hash.clone(),
            source_utf8: FIXTURE.to_owned(),
            anchors,
            unresolved_syntax: unresolved_syntax.clone(),
        });
        closure_files.push(SourceClosureFile {
            path,
            sha256: source_hash.clone(),
            incoming_edges: Vec::new(),
            classification: "fixture".into(),
            classification_rationale: "fixed test source".into(),
        });
    }
    let root = ManifestRoot {
        schema_version: "1".into(),
        katana_revision: FIXED_KATANA_REVISION.into(),
        katana_tree_fingerprint: "tree".into(),
        katana_external_ui_fingerprint: "external".into(),
        user_mandated_extensions_fingerprint: "extensions".into(),
        source_universe_fingerprint: "universe".into(),
        requirement_source_aliases_fingerprint: sha256_hex(ALIAS_LEDGER.as_bytes()),
        kle_tree_fingerprint: "kle".into(),
        kuc_tree_fingerprint: "kuc".into(),
        release_profile_matrix_fingerprint: "profiles".into(),
        generator_fingerprint: "generator".into(),
        generated_at_utc: "fixture".into(),
        static_leaf_count: None,
        expected_leaf_ids: None,
    };
    let source = SourceClosureArtifact {
        root: root.clone(),
        profiles: Vec::new(),
        files: closure_files,
        external_ui_semantic_dependencies: Vec::new(),
        unresolved_edges: Vec::new(),
        source_derived_native_target: None,
        context_menu_target_manifest: None,
    };
    let source_bytes = serde_json::to_vec(&source).map_err(|error| error.to_string())?;
    let catalog = SourceAnchorCatalog {
        schema: SCHEMA.into(),
        source_closure_sha256: sha256_hex(&source_bytes),
        root,
        alias_ledger_sha256: sha256_hex(ALIAS_LEDGER.as_bytes()),
        files,
    };
    Ok((source, source_bytes, catalog))
}

fn reconstruct(catalog: &SourceAnchorCatalog) -> Result<SourceAnchorCatalog, String> {
    let (source, source_bytes, _) = fixture_catalog()?;
    expected_from_embedded_source(&source, &source_bytes, &catalog.files)
}

fn same_catalog(left: &SourceAnchorCatalog, right: &SourceAnchorCatalog) -> Result<bool, String> {
    Ok(
        serde_json::to_value(left).map_err(|error| error.to_string())?
            == serde_json::to_value(right).map_err(|error| error.to_string())?,
    )
}

#[test]
fn catalog_reconstruction_rejects_removed_added_and_mutated_records() -> Result<(), String> {
    let (_, _, catalog) = fixture_catalog()?;
    assert!(same_catalog(&reconstruct(&catalog)?, &catalog)?);

    let mut removed = catalog.clone();
    removed.files.pop();
    assert!(reconstruct(&removed).is_err());

    let mut added = catalog.clone();
    added.files.push(added.files[0].clone());
    assert!(reconstruct(&added).is_err());

    let mut changed = catalog.clone();
    let anchor = changed.files[0]
        .anchors
        .iter_mut()
        .find(|anchor| anchor.kind == AnchorKind::EnumVariant)
        .ok_or("fixture has no enum anchor")?;
    anchor.span.start_byte += 1;
    assert!(!same_catalog(&reconstruct(&changed)?, &changed)?);

    let mut node_hash_changed = catalog.clone();
    node_hash_changed.files[0].anchors[0].node_sha256 = "sha256:tampered".into();
    assert!(!same_catalog(
        &reconstruct(&node_hash_changed)?,
        &node_hash_changed
    )?);

    let mut excerpt_hash_changed = catalog.clone();
    excerpt_hash_changed.files[0].anchors[0].excerpt_sha256 = "sha256:tampered".into();
    assert!(!same_catalog(
        &reconstruct(&excerpt_hash_changed)?,
        &excerpt_hash_changed
    )?);

    let mut duplicate = catalog.clone();
    let repeated_anchor = duplicate.files[0].anchors[0].clone();
    duplicate.files[0].anchors.push(repeated_anchor);
    assert!(!same_catalog(&reconstruct(&duplicate)?, &duplicate)?);
    Ok(())
}

#[test]
fn generation_alias_facts_roundtrip_through_authenticated_reconstruction() -> Result<(), String> {
    let (_, _, catalog) = fixture_catalog()?;
    assert!(
        catalog
            .files
            .iter()
            .flat_map(|file| &file.anchors)
            .all(|anchor| !anchor.requirement_id.is_empty())
    );
    assert!(same_catalog(&reconstruct(&catalog)?, &catalog)?);
    Ok(())
}

#[test]
fn reconstruction_rejects_embedded_source_root_and_alias_tampering() -> Result<(), String> {
    let (_, _, catalog) = fixture_catalog()?;

    let mut source_changed = catalog.clone();
    source_changed.files[0].source_utf8.push('x');
    assert!(reconstruct(&source_changed).is_err());

    let mut root_changed = catalog.clone();
    root_changed.root.katana_revision = "different".into();
    assert!(!same_catalog(&reconstruct(&root_changed)?, &root_changed)?);

    let mut alias_changed = catalog.clone();
    alias_changed.alias_ledger_sha256 = "different".into();
    assert!(!same_catalog(
        &reconstruct(&alias_changed)?,
        &alias_changed
    )?);
    Ok(())
}

#[test]
fn catalog_deserialization_rejects_unknown_fields() -> Result<(), String> {
    let (_, _, catalog) = fixture_catalog()?;
    let mut value = serde_json::to_value(catalog).map_err(|error| error.to_string())?;
    value["unexpected"] = serde_json::Value::Bool(true);
    assert!(serde_json::from_value::<SourceAnchorCatalog>(value).is_err());
    Ok(())
}
