use super::{AliasEntry, RequirementSourceInventoryReport, source};

#[test]
fn distinct_raw_and_normalized_paths_survive_reversed_input_order() -> Result<(), String> {
    let requirements = "## Editor Parity Requirements\n\
| `one` | C/M | views/app_frame/{a,missing}.rs views/app_frame/** | visible |\n\
### Non-Aggregate Authoring Leaf Catalog\n";
    let alias = AliasEntry {
        requirement_id: "one".into(),
        short_reference: "crates/katana-ui/src/views/app_frame/a.rs".into(),
        source_path: "crates/katana-ui/src/views/app_frame/c.rs".into(),
    };
    let paths = [
        "crates/katana-ui/src/views/app_frame/c.rs",
        "crates/katana-ui/src/views/app_frame/a.rs",
    ];
    let first = RequirementSourceInventoryReport::build(
        requirements.as_bytes(),
        &source(&paths),
        std::slice::from_ref(&alias),
    )?;
    let second = RequirementSourceInventoryReport::build(
        requirements.as_bytes(),
        &source(&[paths[1], paths[0]]),
        std::slice::from_ref(&alias),
    )?;
    assert_eq!(first.rows[0].references, second.rows[0].references);
    let references = &first.rows[0].references;
    assert_eq!(references.len(), 5);
    let prefix = "crates/katana-ui/src/views/app_frame/";
    for (raw_suffix, path_suffix, status) in [
        ("**", "a.rs", "resolved"),
        ("**", "c.rs", "resolved"),
        ("a.rs", "a.rs", "resolved"),
        ("a.rs", "c.rs", "resolved"),
        ("missing.rs", "missing.rs", "unscanned"),
    ] {
        let raw_reference = format!("{prefix}{raw_suffix}");
        let normalized_path = format!("{prefix}{path_suffix}");
        assert!(references.iter().any(|reference| {
            reference.raw_reference == raw_reference
                && reference.normalized_path.as_deref() == Some(normalized_path.as_str())
                && reference.status == status
        }));
    }
    assert!(first.rows[0].unresolved.iter().any(
        |reason| reason == "unscanned source: crates/katana-ui/src/views/app_frame/missing.rs"
    ));
    Ok(())
}
