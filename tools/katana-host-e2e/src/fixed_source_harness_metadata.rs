use std::{fs, io, path::Path, process::Output};

use super::fixed_source_harness_error::FixedSourceHarnessError;
use super::fixed_source_harness_types::{FixedSourceHarness, FixedSourceHarnessMetadata};
use crate::system::ProcessService;
use sha2::{Digest, Sha256};

impl FixedSourceHarness {
    pub fn resolve_metadata(&self) -> Result<FixedSourceHarnessMetadata, FixedSourceHarnessError> {
        let directory =
            self.manifest_path
                .parent()
                .ok_or_else(|| FixedSourceHarnessError::WriteMetadata {
                    path: self.manifest_path.clone(),
                    source: io::Error::new(io::ErrorKind::InvalidInput, "manifest has no parent"),
                })?;
        let manifest_path = self.manifest_path.as_os_str().to_string_lossy();
        run_cargo(
            directory,
            "cargo generate-lockfile --manifest-path",
            [
                "generate-lockfile",
                "--manifest-path",
                manifest_path.as_ref(),
            ],
        )?;
        let metadata = run_cargo(
            directory,
            "cargo metadata --locked --format-version 1 --manifest-path",
            [
                "metadata",
                "--locked",
                "--format-version",
                "1",
                "--manifest-path",
                manifest_path.as_ref(),
            ],
        )?;
        let metadata_path = directory.join("cargo-metadata.json");
        let mut metadata_json = serde_json::from_slice::<serde_json::Value>(&metadata.stdout)
            .map_err(|source| FixedSourceHarnessError::InvalidMetadataJson {
                path: metadata_path.clone(),
                source,
            })?;
        fs::write(&metadata_path, &metadata.stdout).map_err(|source| {
            FixedSourceHarnessError::WriteMetadata {
                path: metadata_path.clone(),
                source,
            }
        })?;
        let canonical_metadata =
            canonical_metadata_bytes(&mut metadata_json, directory, &metadata_path)?;
        let cargo_lock_path = directory.join("Cargo.lock");
        let cargo_lock = fs::read(&cargo_lock_path).map_err(|source| {
            FixedSourceHarnessError::ReadCargoLock {
                path: cargo_lock_path,
                source,
            }
        })?;
        Ok(FixedSourceHarnessMetadata {
            metadata_path,
            metadata_sha256: sha256_hex(&canonical_metadata),
            cargo_lock_sha256: sha256_hex(&cargo_lock),
        })
    }
}

fn run_cargo<const N: usize>(
    directory: &Path,
    command: &'static str,
    args: [&str; N],
) -> Result<Output, FixedSourceHarnessError> {
    let output = ProcessService::create_command("cargo")
        .args(args)
        .current_dir(directory)
        .output()
        .map_err(|source| FixedSourceHarnessError::CargoCommand { command, source })?;
    if output.status.success() {
        return Ok(output);
    }
    Err(FixedSourceHarnessError::CargoFailure {
        command,
        status: output.status.code(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let hex = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("sha256:{hex}")
}

const TEMPORARY_HARNESS_MARKER: &str = "<fixed-host-e2e-harness>";

fn canonical_metadata_bytes(
    metadata: &mut serde_json::Value,
    harness_directory: &Path,
    metadata_path: &Path,
) -> Result<Vec<u8>, FixedSourceHarnessError> {
    let prefix = harness_directory.to_string_lossy();
    normalize_string_leaves(metadata, &prefix);
    serde_json::to_vec(metadata).map_err(|source| {
        FixedSourceHarnessError::CanonicalizeMetadataJson {
            path: metadata_path.to_path_buf(),
            source,
        }
    })
}

fn normalize_string_leaves(value: &mut serde_json::Value, prefix: &str) {
    match value {
        serde_json::Value::Array(values) => {
            for value in values {
                normalize_string_leaves(value, prefix);
            }
        }
        serde_json::Value::Object(values) => {
            for value in values.values_mut() {
                normalize_string_leaves(value, prefix);
            }
        }
        serde_json::Value::String(value) => {
            if let Some(normalized) = normalize_harness_path(value, prefix) {
                *value = normalized;
            }
        }
        serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) => {}
    }
}

fn normalize_harness_path(value: &str, prefix: &str) -> Option<String> {
    if prefix.is_empty() {
        return None;
    }
    let mut normalized = String::with_capacity(value.len());
    let mut cursor = 0;
    let mut replaced = false;
    while let Some(relative_start) = value[cursor..].find(prefix) {
        let start = cursor + relative_start;
        let remainder = &value[start + prefix.len()..];
        let path_boundary = remainder.is_empty()
            || remainder.starts_with(std::path::MAIN_SEPARATOR)
            || remainder.starts_with('/')
            || remainder.starts_with('\\');
        let uri_boundary = start == 0
            || value[..start].ends_with("file://")
            || value[..start].ends_with("file:///");
        let uri_path_boundary = uri_boundary
            && (path_boundary || remainder.starts_with('#') || remainder.starts_with('?'));
        if uri_path_boundary {
            normalized.push_str(&value[cursor..start]);
            normalized.push_str(TEMPORARY_HARNESS_MARKER);
            cursor = start + prefix.len();
            replaced = true;
        } else {
            normalized.push_str(&value[cursor..start + prefix.len()]);
            cursor = start + prefix.len();
        }
    }
    if !replaced {
        return None;
    }
    normalized.push_str(&value[cursor..]);
    Some(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_hashes_are_prefixed_sha256_digests() {
        assert_eq!(
            sha256_hex(b"metadata"),
            "sha256:45447b7afbd5e544f7d0f1df0fccd26014d9850130abd3f020b89ff96b82079f"
        );
    }

    #[test]
    fn canonical_metadata_hash_is_stable_across_temporary_roots() {
        let mut first = serde_json::json!({
            "workspace_root": "/tmp/harness-one",
            "target_directory": "/tmp/harness-one/target",
            "root_id": "path+file:///tmp/harness-one#katana-host-e2e-fixed@0.1.0",
            "source": "/readonly/katana/crates/katana-core",
            "note": "prefix-is-not-a-path: /tmp/unrelated-suffix"
        });
        let mut second = serde_json::json!({
            "workspace_root": "/tmp/harness-two",
            "target_directory": "/tmp/harness-two/target",
            "root_id": "path+file:///tmp/harness-two#katana-host-e2e-fixed@0.1.0",
            "source": "/readonly/katana/crates/katana-core",
            "note": "prefix-is-not-a-path: /tmp/unrelated-suffix"
        });
        let first_bytes = canonical_metadata_bytes(
            &mut first,
            Path::new("/tmp/harness-one"),
            Path::new("/tmp/first-metadata.json"),
        )
        .expect("canonical JSON should serialize");
        let second_bytes = canonical_metadata_bytes(
            &mut second,
            Path::new("/tmp/harness-two"),
            Path::new("/tmp/second-metadata.json"),
        )
        .expect("canonical JSON should serialize");

        assert_eq!(first, second);
        assert_eq!(sha256_hex(&first_bytes), sha256_hex(&second_bytes));
        assert_eq!(first["source"], second["source"]);
        assert_eq!(first["note"], "prefix-is-not-a-path: /tmp/unrelated-suffix");
    }

    #[test]
    fn canonical_metadata_hash_changes_when_source_path_changes() {
        let mut first = serde_json::json!({"source": "/readonly/katana/crates/katana-core"});
        let mut second = serde_json::json!({"source": "/readonly/katana/crates/katana-ui"});
        let first_bytes = canonical_metadata_bytes(
            &mut first,
            Path::new("/tmp/harness-one"),
            Path::new("/tmp/first-metadata.json"),
        )
        .expect("canonical JSON should serialize");
        let second_bytes = canonical_metadata_bytes(
            &mut second,
            Path::new("/tmp/harness-one"),
            Path::new("/tmp/second-metadata.json"),
        )
        .expect("canonical JSON should serialize");

        assert_ne!(sha256_hex(&first_bytes), sha256_hex(&second_bytes));
    }

    #[test]
    fn canonical_metadata_does_not_replace_a_partial_prefix() {
        let mut metadata = serde_json::json!({
            "value": "/tmp/harness-one-suffix/target",
            "non_path": "prefix /tmp/harness-one/target"
        });
        let bytes = canonical_metadata_bytes(
            &mut metadata,
            Path::new("/tmp/harness-one"),
            Path::new("/tmp/metadata.json"),
        )
        .expect("canonical JSON should serialize");

        assert_eq!(
            bytes,
            br#"{"non_path":"prefix /tmp/harness-one/target","value":"/tmp/harness-one-suffix/target"}"#
        );
    }
}
