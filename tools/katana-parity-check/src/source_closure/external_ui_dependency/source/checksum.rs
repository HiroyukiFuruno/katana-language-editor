use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use serde_json::Value;

use super::path_validation;

const SHA256_HEX_LENGTH: usize = 64;

pub(super) fn optional_file_hashes(
    root: &Path,
    expected_package: &str,
) -> Result<Option<BTreeMap<String, String>>, String> {
    let path = root.join(".cargo-checksum.json");
    match fs::symlink_metadata(&path) {
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "egui source checksum evidence is unavailable: {error}"
            ));
        }
        Ok(_) => {}
    }
    let path = path_validation::checksum_manifest(root)?;
    let bytes = fs::read(path)
        .map_err(|error| format!("egui source checksum evidence is unavailable: {error}"))?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("egui source checksum manifest is malformed: {error}"))?;
    if value.get("package").and_then(Value::as_str) != Some(expected_package) {
        return Err("egui source checksum differs from Cargo.lock checksum".into());
    }
    let files = value
        .get("files")
        .and_then(Value::as_object)
        .ok_or_else(|| "egui source checksum manifest has no files object".to_owned())?;
    let mut hashes = BTreeMap::new();
    for (relative, value) in files {
        path_validation::source_path(root, relative)?;
        let hash = value
            .as_str()
            .ok_or_else(|| format!("egui source checksum manifest hash is invalid: {relative}"))?;
        if !is_sha256(hash) {
            return Err(format!(
                "egui source checksum manifest hash is invalid: {relative}"
            ));
        }
        hashes.insert(relative.clone(), hash.to_owned());
    }
    Ok(Some(hashes))
}

fn is_sha256(value: &str) -> bool {
    value.len() == SHA256_HEX_LENGTH && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
