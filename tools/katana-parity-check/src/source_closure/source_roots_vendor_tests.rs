use std::collections::BTreeSet;
use std::fs;

use crate::capability_manifest::capability_manifest_test_fixtures::FixtureBuilder;

#[test]
fn documented_vendor_input_source_cannot_be_omitted() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = FixtureBuilder::root()?;
    let relative = "vendor/egui-winit/src/lib.rs";
    fs::create_dir_all(fixture.path().join("vendor/egui-winit/src"))?;
    fs::write(fixture.path().join(relative), "fn platform_input() {}")?;
    let documented = format!("Fixed platform bridge: `{relative}`.");
    assert!(matches!(
        super::validate_documented_paths(documented.as_bytes(), fixture.path(), &BTreeSet::new()),
        Err(reason) if reason.contains(relative)
    ));
    super::validate_documented_paths(
        documented.as_bytes(),
        fixture.path(),
        &BTreeSet::from([relative.into()]),
    )?;
    super::validate_documented_file_roots(documented.as_bytes(), &[relative.into()])?;
    Ok(())
}
