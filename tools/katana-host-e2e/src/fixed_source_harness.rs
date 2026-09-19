use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::system::ProcessService;

#[path = "fixed_source_harness_metadata.rs"]
mod fixed_source_harness_metadata;
#[path = "fixed_source_harness_error.rs"]
mod fixed_source_harness_error;
#[path = "fixed_source_harness_types.rs"]
mod fixed_source_harness_types;
pub use fixed_source_harness_error::FixedSourceHarnessError;
pub use fixed_source_harness_types::{
    FixedSourceHarness, FixedSourceHarnessBuilder, FixedSourceHarnessMetadata, FixedSourceHarnessRequest,
};

pub(crate) const FIXED_KATANA_REVISION: &str = "4f6a6287c650a38633c7baeb544a92e739c68567";

impl FixedSourceHarnessBuilder {
    pub fn required_katana_repo() -> Result<PathBuf, FixedSourceHarnessError> {
        std::env::var_os("KATANA_REPO").map(PathBuf::from).ok_or(
            FixedSourceHarnessError::MissingEnvironment {
                variable: "KATANA_REPO",
            },
        )
    }

    pub fn new(request: FixedSourceHarnessRequest) -> Self {
        Self { request }
    }

    pub fn build(self) -> Result<FixedSourceHarness, FixedSourceHarnessError> {
        let katana_source = Self::canonicalize(&self.request.katana_source)?;
        let kle_source = Self::canonicalize(&self.request.kle_source)?;
        let kuc_source = Self::canonicalize(&self.request.kuc_source)?;
        let profile = Self::canonicalize(&self.request.source_closure_profile)?;
        let revision = Self::git_output(&katana_source, "rev-parse HEAD")?;
        Self::validate_head(&revision)?;
        let status = Self::git_output(&katana_source, "status --porcelain")?;
        Self::validate_worktree(&status)?;
        Self::verify_profile(&profile)?;
        let directory = Self::create_unique_directory()?;
        let manifest_path = directory.join("Cargo.toml");
        fs::write(
            &manifest_path,
            Self::manifest(&katana_source, &kle_source, &kuc_source),
        )
        .map_err(|source| FixedSourceHarnessError::WriteManifest {
            path: manifest_path.clone(),
            source,
        })?;
        Ok(FixedSourceHarness {
            manifest_path,
            katana_source,
            kle_source,
            kuc_source,
            source_closure_profile: profile,
            katana_revision: revision,
        })
    }

    fn canonicalize(path: &Path) -> Result<PathBuf, FixedSourceHarnessError> {
        path.canonicalize()
            .map_err(|source| FixedSourceHarnessError::Canonicalize {
                path: path.to_path_buf(),
                source,
            })
    }

    fn validate_head(actual: &str) -> Result<(), FixedSourceHarnessError> {
        if actual == FIXED_KATANA_REVISION {
            return Ok(());
        }
        Err(FixedSourceHarnessError::HeadMismatch {
            expected: FIXED_KATANA_REVISION,
            actual: actual.to_owned(),
        })
    }

    fn validate_worktree(status: &str) -> Result<(), FixedSourceHarnessError> {
        if status.is_empty() {
            return Ok(());
        }
        Err(FixedSourceHarnessError::DirtyWorktree {
            status: status.to_owned(),
        })
    }

    fn git_output(root: &Path, command: &'static str) -> Result<String, FixedSourceHarnessError> {
        let output = ProcessService::create_command("git")
            .args(command.split_ascii_whitespace())
            .current_dir(root)
            .output()
            .map_err(|source| FixedSourceHarnessError::GitCommand { command, source })?;
        if !output.status.success() {
            return Err(FixedSourceHarnessError::GitFailure {
                command,
                status: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            });
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    }

    fn verify_profile(path: &Path) -> Result<(), FixedSourceHarnessError> {
        let bytes = fs::read(path).map_err(|source| FixedSourceHarnessError::ReadProfile {
            path: path.to_path_buf(),
            source,
        })?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|source| {
            FixedSourceHarnessError::InvalidProfileJson {
                path: path.to_path_buf(),
                source,
            }
        })?;
        let revision = value
            .get("root")
            .and_then(|root| root.get("katana_revision"))
            .and_then(serde_json::Value::as_str)
            .or_else(|| {
                value
                    .get("katana_revision")
                    .and_then(serde_json::Value::as_str)
            })
            .ok_or_else(|| FixedSourceHarnessError::MissingProfileRevision {
                path: path.to_path_buf(),
            })?;
        if revision != FIXED_KATANA_REVISION {
            return Err(FixedSourceHarnessError::ProfileRevisionMismatch {
                expected: FIXED_KATANA_REVISION,
                actual: revision.to_owned(),
            });
        }
        Ok(())
    }

    fn create_unique_directory() -> Result<PathBuf, FixedSourceHarnessError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|source| FixedSourceHarnessError::SystemTime { source })?
            .as_nanos();
        let base = std::env::temp_dir();
        for attempt in 0..100_u32 {
            let path = base.join(format!(
                "kle-fixed-host-e2e-{timestamp}-{}-{attempt}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(path),
                Err(source) if source.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(source) => {
                    return Err(FixedSourceHarnessError::CreateHarnessDirectory { path, source });
                }
            }
        }
        let path = base.join(format!(
            "kle-fixed-host-e2e-{timestamp}-{}-exhausted",
            std::process::id()
        ));
        Err(FixedSourceHarnessError::CreateHarnessDirectory {
            path,
            source: io::Error::new(
                io::ErrorKind::AlreadyExists,
                "unique directory attempts exhausted",
            ),
        })
    }

    fn manifest(katana: &Path, kle: &Path, _kuc: &Path) -> String {
        format!(
            "[package]\nname = \"katana-host-e2e-fixed\"\nversion = \"0.1.0\"\nedition = \"2024\"\npublish = false\n\n[features]\ndefault = []\nfixed-host = [\"dep:katana_ui\", \"dep:katana_core\", \"dep:katana_platform\"]\n\n[lib]\npath = \"{}/tools/katana-host-e2e/src/lib.rs\"\n\n[[example]]\nname = \"initial_frame\"\npath = \"{}/tools/katana-host-e2e/examples/initial_frame.rs\"\n\n[dependencies]\neframe36 = {{ package = \"eframe\", version = \"0.36\" }}\naccesskit = \"0.24\"\nsha2 = \"0.11\"\nserde_json = \"1\"\nkatana_ui = {{ package = \"katana-ui\", path = \"{}/crates/katana-ui\", optional = true }}\nkatana_core = {{ package = \"katana-core\", path = \"{}/crates/katana-core\", optional = true }}\nkatana_platform = {{ package = \"katana-platform\", path = \"{}/crates/katana-platform\", optional = true }}\n\n[patch.crates-io]\negui_commonmark = {{ path = \"{}/vendor/egui_commonmark_upstream/egui_commonmark\" }}\negui_commonmark_backend = {{ path = \"{}/vendor/egui_commonmark_upstream/egui_commonmark_backend\" }}\negui-winit = {{ path = \"{}/vendor/egui-winit\" }}\nmathjax_svg = {{ path = \"{}/vendor/mathjax_svg\" }}\n",
            kle.display(),
            kle.display(),
            katana.display(),
            katana.display(),
            katana.display(),
            katana.display(),
            katana.display(),
            katana.display(),
            katana.display(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_profile_with_wrong_revision() {
        let path = std::env::temp_dir().join(format!("kle-profile-{}", std::process::id()));
        assert!(std::fs::write(&path, r#"{"katana_revision":"wrong"}"#).is_ok());
        let result = FixedSourceHarnessBuilder::verify_profile(&path);
        assert!(matches!(
            result,
            Err(FixedSourceHarnessError::ProfileRevisionMismatch { .. })
        ));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn rejects_profile_without_revision() {
        let path = std::env::temp_dir().join(format!("kle-profile-missing-{}", std::process::id()));
        assert!(std::fs::write(&path, r#"{"profile_id":"macos-latest"}"#).is_ok());
        let result = FixedSourceHarnessBuilder::verify_profile(&path);
        assert!(matches!(
            result,
            Err(FixedSourceHarnessError::MissingProfileRevision { .. })
        ));
        assert!(std::fs::remove_file(path).is_ok());
    }

    #[test]
    fn rejects_head_and_dirty_worktree() {
        let head = FixedSourceHarnessBuilder::validate_head("mutable-branch");
        assert!(matches!(
            head,
            Err(FixedSourceHarnessError::HeadMismatch { .. })
        ));
        let dirty = FixedSourceHarnessBuilder::validate_worktree(" M Cargo.toml");
        assert!(matches!(
            dirty,
            Err(FixedSourceHarnessError::DirtyWorktree { .. })
        ));
    }

    #[test]
    fn generated_manifest_uses_fixed_absolute_sources() {
        let manifest = FixedSourceHarnessBuilder::manifest(
            Path::new("/readonly/katana"),
            Path::new("/readonly/kle"),
            Path::new("/readonly/kuc"),
        );
        assert!(manifest.contains("/readonly/katana/crates/katana-ui"));
        assert!(manifest.contains("/readonly/kle/tools/katana-host-e2e/src/lib.rs"));
        assert!(manifest.contains("fixed-host = ["));
        assert!(manifest.contains("optional = true"));
        assert!(!manifest.contains("katana-language-editor"));
        assert!(!manifest.contains("katana-ui-core"));
        assert!(!manifest.contains("/readonly/kuc"));
        assert!(!manifest.contains("../../../katana"));
    }

    #[test]
    fn cargo_failure_is_typed_without_starting_a_process() {
        let error = FixedSourceHarnessError::CargoFailure {
            command: "cargo metadata",
            status: Some(101),
            stderr: "manifest failed".to_owned(),
        };
        assert!(matches!(
            error,
            FixedSourceHarnessError::CargoFailure {
                command: "cargo metadata",
                status: Some(101),
                ..
            }
        ));
    }
}
