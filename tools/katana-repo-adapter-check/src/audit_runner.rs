use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::system::ProcessService;

const RUNNABLE_TEST_ARGUMENT_COUNT: usize = 10;

pub(crate) struct KatanaRunnableAudit<'a> {
    repo_root: &'a Path,
}

impl<'a> KatanaRunnableAudit<'a> {
    pub(crate) fn new(repo_root: &'a Path) -> Self {
        Self { repo_root }
    }

    pub(crate) fn validate(&self, test_names: &[&str]) -> Result<(), String> {
        let before = SourceSnapshot::capture(self.repo_root)?;
        for test_name in test_names {
            self.validate_test(test_name)?;
        }
        let after = SourceSnapshot::capture(self.repo_root)?;
        if before == after {
            return Ok(());
        }
        Err("KatanA reference checkout changed during read-only validation".to_string())
    }

    fn validate_test(&self, test_name: &str) -> Result<(), String> {
        let target_dir = TemporaryTargetDir::new("katana-repo-adapter-check-target")?;
        let output = ProcessService::create_command("cargo")
            .current_dir(self.repo_root)
            .env("CARGO_TARGET_DIR", target_dir.path())
            .args(Self::test_command_arguments(test_name))
            .output()
            .map_err(|error| {
                format!("failed to launch runnable KatanA test `{test_name}`: {error}")
            })?;
        if !output.status.success() {
            return Err(format!(
                "KatanA test target `{test_name}` is not runnable: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains(&format!("test {test_name} ... ok")) {
            return Ok(());
        }
        Err(format!(
            "KatanA test target `{test_name}` did not execute the expected exact test"
        ))
    }

    fn test_command_arguments(test_name: &str) -> [&str; RUNNABLE_TEST_ARGUMENT_COUNT] {
        [
            "test",
            "--locked",
            "-p",
            "katana-ui",
            "--test",
            "ui_integration_parallel",
            test_name,
            "--",
            "--exact",
            "--nocapture",
        ]
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SourceSnapshot(BTreeMap<PathBuf, Vec<u8>>);

impl SourceSnapshot {
    pub(crate) fn capture(root: &Path) -> Result<Self, String> {
        let mut files = BTreeMap::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(directory) = stack.pop() {
            Self::collect_directory(root, &directory, &mut stack, &mut files)?;
        }
        Ok(Self(files))
    }

    fn collect_directory(
        root: &Path,
        directory: &Path,
        stack: &mut Vec<PathBuf>,
        files: &mut BTreeMap<PathBuf, Vec<u8>>,
    ) -> Result<(), String> {
        let entries = fs::read_dir(directory)
            .map_err(|error| format!("failed to inspect {}: {error}", directory.display()))?;
        for entry in entries {
            let entry = entry.map_err(|error| error.to_string())?;
            let path = entry.path();
            let file_type = entry.file_type().map_err(|error| error.to_string())?;
            if file_type.is_dir() {
                Self::collect_directory_path(path, stack);
                continue;
            }
            if file_type.is_file() {
                Self::collect_file(root, path, files)?;
            }
        }
        Ok(())
    }

    fn collect_directory_path(path: PathBuf, stack: &mut Vec<PathBuf>) {
        if path
            .file_name()
            .is_some_and(|name| name == ".git" || name == "target")
        {
            return;
        }
        stack.push(path);
    }

    fn collect_file(
        root: &Path,
        path: PathBuf,
        files: &mut BTreeMap<PathBuf, Vec<u8>>,
    ) -> Result<(), String> {
        let relative = path
            .strip_prefix(root)
            .map_err(|error| error.to_string())?
            .to_path_buf();
        let contents = fs::read(&path).map_err(|error| error.to_string())?;
        files.insert(relative, contents);
        Ok(())
    }
}

struct TemporaryTargetDir {
    path: PathBuf,
}

impl TemporaryTargetDir {
    fn new(prefix: &str) -> Result<Self, String> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos();
        Ok(Self {
            path: std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id())),
        })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporaryTargetDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::KatanaRunnableAudit;

    #[test]
    fn runnable_command_executes_the_exact_named_test_without_listing() {
        let arguments = KatanaRunnableAudit::test_command_arguments(
            "editor_kle_downstream_adapter::kle_event_action_stream_updates_real_katana_editor_state",
        );

        assert!(arguments.contains(&"--exact"));
        assert!(arguments.contains(&"--nocapture"));
        assert!(!arguments.contains(&"--list"));
        assert_eq!(
            arguments[6],
            "editor_kle_downstream_adapter::kle_event_action_stream_updates_real_katana_editor_state"
        );
    }
}
