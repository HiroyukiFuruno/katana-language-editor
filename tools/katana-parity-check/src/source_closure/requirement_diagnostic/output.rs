use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use serde::Serialize;

pub(super) fn validate_output_location(output: &Path, katana_root: &Path) -> Result<(), String> {
    if output.exists() {
        return Err(format!(
            "diagnostic output already exists: {}",
            output.display()
        ));
    }
    let parent = output
        .parent()
        .ok_or_else(|| "diagnostic output has no parent".to_string())?
        .canonicalize()
        .map_err(|error| format!("diagnostic output parent is missing or unreadable: {error}"))?;
    if !parent.is_dir() {
        return Err(format!(
            "diagnostic output parent is not a directory: {}",
            parent.display()
        ));
    }
    let canonical_root = katana_root
        .canonicalize()
        .map_err(|error| format!("KatanA checkout is unreadable: {error}"))?;
    if parent.starts_with(canonical_root) {
        return Err("diagnostic output must be outside the fixed KatanA checkout".into());
    }
    Ok(())
}

pub(super) fn write_new_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "diagnostic output has no parent".to_string())?;
    if !parent.is_dir() {
        return Err(format!(
            "diagnostic output parent does not exist: {}",
            parent.display()
        ));
    }
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("serialize diagnostic output: {error}"))?;
    bytes.push(b'\n');
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("create diagnostic output: {error}"))?;
    file.write_all(&bytes)
        .map_err(|error| format!("write diagnostic output: {error}"))
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::fs::symlink;

    use super::{validate_output_location, write_new_json};
    use crate::capability_manifest::capability_manifest_test_fixtures::FixtureBuilder;

    #[test]
    fn output_cannot_follow_a_link_back_into_checkout() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = FixtureBuilder::root()?;
        let root = fixture.path().canonicalize()?;
        let checkout = root.join("katana");
        std::fs::create_dir(&checkout)?;
        let linked_parent = root.join("linked");
        symlink(&checkout, &linked_parent)?;
        let output = linked_parent.join("diagnostic.json");
        assert!(validate_output_location(&output, &checkout).is_err());
        assert!(!checkout.join("diagnostic.json").exists());

        let absent_target = root.join("absent.json");
        let output = root.join("dangling.json");
        symlink(&absent_target, &output)?;
        assert!(write_new_json(&output, &"diagnostic").is_err());
        assert!(!absent_target.exists());
        Ok(())
    }
}
