use super::source_requirement_alias_ledger::{AliasEntry, RequirementSourceAliasLedger};

const REQUIREMENTS: &str = "## Editor Parity Requirements\n| ID | Status | KatanA decisive branch |\n| --- | --- | --- |\n| `editor.one` | C/M | `text_edit.rs`, `ui.rs` |\n### Non-Aggregate Authoring Leaf Catalog\n";
const UNIVERSE: &str = "crates/katana-ui/src/views/panels/editor/**\n";
const MANIFEST: &str =
    r#"{"directory_roots":["crates/katana-ui/src/views/panels/editor"],"file_roots":[]}"#;
const VALID: &str = r#"
{
  "schema_version":"1",
  "katana_revision":"4f6a6287c650a38633c7baeb544a92e739c68567",
  "entries":[
    {"requirement_id":"editor.one","short_reference":"text_edit.rs","source_path":"crates/katana-ui/src/views/panels/editor/text_edit.rs"},
    {"requirement_id":"editor.one","short_reference":"ui.rs","source_path":"crates/katana-ui/src/views/panels/editor/ui.rs"}
  ],
  "non_katana_references":[]
}
"#;

#[test]
fn aliases_require_exact_requirement_classification_and_source_root_coverage()
-> Result<(), Box<dyn std::error::Error>> {
    RequirementSourceAliasLedger::validate_coverage(REQUIREMENTS, UNIVERSE, MANIFEST, VALID)
        .map_err(std::io::Error::other)?;
    let mut missing: serde_json::Value = serde_json::from_str(VALID)?;
    missing["entries"]
        .as_array_mut()
        .ok_or("entries must be an array")?
        .retain(|entry| entry["short_reference"] != "ui.rs");
    assert!(matches!(
        RequirementSourceAliasLedger::validate_coverage(
            REQUIREMENTS,
            UNIVERSE,
            MANIFEST,
            &serde_json::to_string(&missing)?,
        ),
        Err(error) if error.contains("unclassified requirement source: editor.one:ui.rs")
    ));
    let direct_universe = "crates/katana-ui/src/views/panels/editor/text_edit.rs\ncrates/katana-ui/src/views/panels/editor/ui.rs\n";
    let omitted_root = MANIFEST.replace(
        "crates/katana-ui/src/views/panels/editor",
        "crates/katana-ui/src/views/panels/problems",
    );
    assert!(matches!(
        RequirementSourceAliasLedger::validate_coverage(
            REQUIREMENTS,
            direct_universe,
            &omitted_root,
            VALID,
        ),
        Err(error) if error.contains("absent from source-root manifest")
    ));
    Ok(())
}

#[test]
fn checked_in_alias_ledger_classifies_every_short_requirement_reference()
-> Result<(), Box<dyn std::error::Error>> {
    RequirementSourceAliasLedger::validate_checked_in_coverage().map_err(std::io::Error::other)?;
    Ok(())
}

#[test]
fn checked_in_alias_entries_are_structured_and_deterministic()
-> Result<(), Box<dyn std::error::Error>> {
    let entries =
        RequirementSourceAliasLedger::verified_alias_entries().map_err(std::io::Error::other)?;
    assert_eq!(entries.len(), 70);
    assert_eq!(
        entries.first(),
        Some(&AliasEntry {
            requirement_id: "clipboard.text-copy-empty-selection".into(),
            short_reference: "text_edit.rs".into(),
            source_path: "crates/katana-ui/src/views/panels/editor/text_edit.rs".into(),
        })
    );
    assert_eq!(
        entries.last(),
        Some(&AliasEntry {
            requirement_id: "toolbar.viewport-clamp".into(),
            short_reference: "toolbar_popup.rs".into(),
            source_path: "crates/katana-ui/src/views/panels/editor/toolbar_popup.rs".into(),
        })
    );
    assert!(entries.windows(2).all(|window| {
        (
            &window[0].requirement_id,
            &window[0].short_reference,
            &window[0].source_path,
        ) <= (
            &window[1].requirement_id,
            &window[1].short_reference,
            &window[1].source_path,
        )
    }));
    Ok(())
}

#[test]
fn fixture_alias_entries_are_independent_of_input_order_and_exclude_external_references()
-> Result<(), Box<dyn std::error::Error>> {
    let requirements = REQUIREMENTS.replace(
        "`text_edit.rs`, `ui.rs`",
        "`text_edit.rs`, `ui.rs`, `external.rs`",
    );
    let mut input: serde_json::Value = serde_json::from_str(VALID)?;
    input["non_katana_references"] = serde_json::json!([{
        "requirement_id": "editor.one",
        "short_reference": "external.rs",
        "owner": "KUC",
        "reason": "owned outside KatanA"
    }]);
    let original_entries = RequirementSourceAliasLedger::validated_entries(
        &requirements,
        UNIVERSE,
        MANIFEST,
        &serde_json::to_string(&input)?,
    )
    .map_err(std::io::Error::other)?;
    input["entries"]
        .as_array_mut()
        .ok_or("entries must be an array")?
        .reverse();
    let entries = RequirementSourceAliasLedger::validated_entries(
        &requirements,
        UNIVERSE,
        MANIFEST,
        &serde_json::to_string(&input)?,
    )
    .map_err(std::io::Error::other)?;
    assert_eq!(entries, original_entries);
    assert_eq!(entries.len(), 2);
    assert!(
        entries
            .iter()
            .all(|entry| entry.short_reference != "external.rs")
    );
    assert!(entries.windows(2).all(|window| {
        (
            &window[0].requirement_id,
            &window[0].short_reference,
            &window[0].source_path,
        ) <= (
            &window[1].requirement_id,
            &window[1].short_reference,
            &window[1].source_path,
        )
    }));
    Ok(())
}

#[test]
fn aliases_reject_stale_schema_and_revision() -> Result<(), Box<dyn std::error::Error>> {
    let mut stale_schema: serde_json::Value = serde_json::from_str(VALID)?;
    stale_schema["schema_version"] = serde_json::json!("0");
    assert!(matches!(
        RequirementSourceAliasLedger::validate_coverage(
            REQUIREMENTS,
            UNIVERSE,
            MANIFEST,
            &serde_json::to_string(&stale_schema)?,
        ),
        Err(error) if error.contains("fixed schema/revision")
    ));
    let mut stale_revision: serde_json::Value = serde_json::from_str(VALID)?;
    stale_revision["katana_revision"] =
        serde_json::json!("0000000000000000000000000000000000000000");
    assert!(matches!(
        RequirementSourceAliasLedger::validate_coverage(
            REQUIREMENTS,
            UNIVERSE,
            MANIFEST,
            &serde_json::to_string(&stale_revision)?,
        ),
        Err(error) if error.contains("fixed schema/revision")
    ));
    Ok(())
}

#[test]
fn aliases_reject_duplicate_entries_and_cross_collection_classifications()
-> Result<(), Box<dyn std::error::Error>> {
    let mut duplicate: serde_json::Value = serde_json::from_str(VALID)?;
    let first_entry = duplicate["entries"]
        .as_array()
        .and_then(|entries| entries.first())
        .cloned()
        .ok_or("first entry missing")?;
    duplicate["entries"]
        .as_array_mut()
        .ok_or("entries must be an array")?
        .push(first_entry.clone());
    assert!(matches!(
        RequirementSourceAliasLedger::validate_coverage(
            REQUIREMENTS,
            UNIVERSE,
            MANIFEST,
            &serde_json::to_string(&duplicate)?,
        ),
        Err(error) if error.contains("duplicate alias classification")
    ));

    let mut cross_collection_duplicate: serde_json::Value = serde_json::from_str(VALID)?;
    cross_collection_duplicate["non_katana_references"] = serde_json::json!([{
        "requirement_id": "editor.one",
        "short_reference": "text_edit.rs",
        "owner": "KUC",
        "reason": "test"
    }]);
    assert!(matches!(
        RequirementSourceAliasLedger::validate_coverage(
            REQUIREMENTS,
            UNIVERSE,
            MANIFEST,
            &serde_json::to_string(&cross_collection_duplicate)?,
        ),
        Err(error) if error.contains("duplicate alias classification")
    ));
    Ok(())
}

#[test]
fn aliases_reject_unknown_paths_and_invalid_external_source_classifications()
-> Result<(), Box<dyn std::error::Error>> {
    let mut unknown_path: serde_json::Value = serde_json::from_str(VALID)?;
    unknown_path["entries"][0]["source_path"] =
        serde_json::json!("crates/katana-ui/src/views/panels/unknown.rs");
    assert!(matches!(
        RequirementSourceAliasLedger::validate_coverage(
            REQUIREMENTS,
            UNIVERSE,
            MANIFEST,
            &serde_json::to_string(&unknown_path)?,
        ),
        Err(error) if error.contains("absent from source-universe")
    ));
    let mut external_source: serde_json::Value = serde_json::from_str(VALID)?;
    external_source["non_katana_references"] = serde_json::json!([{
        "requirement_id": "outside",
        "short_reference": "crates/katana-ui/src/views/panels/editor/ui.rs",
        "owner": "KUC",
        "reason": "test"
    }]);
    assert!(matches!(
        RequirementSourceAliasLedger::validate_coverage(
            REQUIREMENTS,
            UNIVERSE,
            MANIFEST,
            &serde_json::to_string(&external_source)?,
        ),
        Err(error) if error.contains("invalid external source classification")
    ));
    Ok(())
}
