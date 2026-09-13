use super::{validate_output_location, verify_checked_in_input, write_new_json};

use crate::capability_manifest::capability_manifest_test_fixtures::FixtureBuilder;
use serde::Serialize;

#[derive(Serialize)]
struct Fixture {
    value: &'static str,
}

#[test]
fn stale_checked_in_input_is_rejected() {
    assert!(verify_checked_in_input("source universe", b"stale", b"compiled").is_err());
}

#[test]
fn registry_identity_is_bound_without_claiming_platform_or_kuc_execution() -> Result<(), String> {
    let fingerprint = "sha256:verified-external-source-audit";
    let root = super::root::diagnostic_root(
        &super::super::scan_state::ScanState::default(),
        "universe",
        "aliases",
        fingerprint,
    )?;
    assert_eq!(root.katana_external_ui_fingerprint, fingerprint);
    assert_eq!(root.kuc_tree_fingerprint, "diagnostic:not-captured");
    assert_eq!(
        root.release_profile_matrix_fingerprint,
        "diagnostic:not-captured"
    );
    Ok(())
}

#[test]
fn existing_output_is_preserved_and_rejected() -> Result<(), String> {
    let fixture = FixtureBuilder::root()?;
    let root = fixture.path();
    let output = root.join("diagnostic.json");
    std::fs::write(&output, b"original\n").map_err(|error| format!("write fixture: {error}"))?;
    let result = write_new_json(&output, &Fixture { value: "changed" });
    let bytes = std::fs::read(&output).map_err(|error| format!("read fixture: {error}"))?;
    assert!(result.is_err());
    assert_eq!(bytes, b"original\n");
    Ok(())
}

#[test]
fn output_inside_checkout_is_rejected_before_creation() -> Result<(), String> {
    let fixture = FixtureBuilder::root()?;
    let root = fixture.path();
    let nested = root.join(".git");
    std::fs::create_dir_all(&nested).map_err(|error| format!("create fixture: {error}"))?;
    let output = nested.join("diagnostic.json");
    let result = validate_output_location(&output, root);
    let created = output.exists();
    assert!(result.is_err());
    assert!(!created);
    Ok(())
}
