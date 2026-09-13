use super::source_requirements_ledger::SourceRequirementsLedger;
use super::source_roots_ledger::{validate_documented_directories, validate_documented_file_roots};

const SOURCE_UNIVERSE: &str =
    include_str!("../../../../docs/v0-1-0-katana-editor-source-universe.md");
const ROOT_MANIFEST: &str = include_str!("../../../../docs/v0-1-0-source-closure-roots.json");
const REQUIREMENTS: &str = include_str!("../../../../docs/v0-1-0-editor-requirements.md");

#[test]
fn checked_in_file_roots_are_all_documented_source_routes() -> Result<(), Box<dyn std::error::Error>>
{
    let manifest: serde_json::Value = serde_json::from_str(ROOT_MANIFEST)?;
    let roots = manifest["file_roots"]
        .as_array()
        .ok_or("source-root manifest file_roots is missing")?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or("source-root manifest file root is not a string")
        })
        .collect::<Result<Vec<_>, _>>()?;

    validate_documented_file_roots(SOURCE_UNIVERSE.as_bytes(), &roots)?;
    Ok(())
}

#[test]
fn checked_in_directory_roots_cover_every_documented_complete_directory()
-> Result<(), Box<dyn std::error::Error>> {
    let manifest: serde_json::Value = serde_json::from_str(ROOT_MANIFEST)?;
    let roots = manifest["directory_roots"]
        .as_array()
        .ok_or("source-root manifest directory_roots is missing")?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or("source-root manifest directory root is not a string")
        })
        .collect::<Result<Vec<_>, _>>()?;

    validate_documented_directories(SOURCE_UNIVERSE.as_bytes(), &roots)?;
    Ok(())
}

#[test]
fn checked_in_source_universe_covers_every_editor_requirement_source()
-> Result<(), Box<dyn std::error::Error>> {
    SourceRequirementsLedger::validate_coverage(
        REQUIREMENTS.as_bytes(),
        SOURCE_UNIVERSE.as_bytes(),
    )?;
    Ok(())
}

#[test]
fn checked_in_root_manifest_uses_the_source_universe_sha256_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let manifest: serde_json::Value = serde_json::from_str(ROOT_MANIFEST)?;
    let actual = manifest["source_universe_sha256"]
        .as_str()
        .ok_or("source-root manifest source_universe_sha256 is missing")?;
    assert_eq!(
        actual,
        super::fingerprint::sha256_hex(SOURCE_UNIVERSE.as_bytes())
    );
    Ok(())
}

#[test]
fn undocumented_file_root_is_rejected() {
    assert!(matches!(
        validate_documented_file_roots(
            b"# KatanA Editor Source Universe\n",
            &["crates/katana-ui/src/app/untracked.rs".to_string()],
        ),
        Err(error) if error.contains("contains undocumented file root")
    ));
}

#[test]
fn workspace_search_requirement_sources_are_closure_roots() -> Result<(), Box<dyn std::error::Error>>
{
    let manifest: serde_json::Value = serde_json::from_str(ROOT_MANIFEST)?;
    let file_roots = manifest["file_roots"]
        .as_array()
        .ok_or("source-root manifest file_roots is missing")?;
    let directory_roots = manifest["directory_roots"]
        .as_array()
        .ok_or("source-root manifest directory_roots is missing")?;
    let modal = "crates/katana-ui/src/views/modals/search.rs";
    let tabs = "crates/katana-ui/src/views/modals/search_tabs";

    assert!(REQUIREMENTS.contains(modal));
    assert!(REQUIREMENTS.contains("views/modals/search_tabs/filename_tab.rs"));
    assert!(REQUIREMENTS.contains("views/modals/search_tabs/md_tab.rs"));
    assert!(SOURCE_UNIVERSE.contains(modal));
    assert!(SOURCE_UNIVERSE.contains("views/modals/search_tabs/**"));
    assert!(file_roots.iter().any(|root| root.as_str() == Some(modal)));
    assert!(
        directory_roots
            .iter()
            .any(|root| root.as_str() == Some(tabs))
    );
    Ok(())
}
