use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) struct FixtureDirectory {
    path: PathBuf,
}

impl FixtureDirectory {
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for FixtureDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub(crate) struct FixtureBuilder;

impl FixtureBuilder {
    pub(crate) fn root() -> Result<FixtureDirectory, String> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos();
        Self::root_with_timestamp(timestamp)
    }

    fn root_with_timestamp(timestamp: u128) -> Result<FixtureDirectory, String> {
        let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "katana-parity-manifest-{}-{timestamp}-{sequence}",
            std::process::id()
        ));
        Self::reserve_path(path)
    }

    fn reserve_path(path: PathBuf) -> Result<FixtureDirectory, String> {
        fs::create_dir(&path).map_err(|error| error.to_string())?;
        Ok(FixtureDirectory { path })
    }

    pub(super) fn write_katana(
        root: &Path,
        registration: &str,
        adapter: &str,
    ) -> Result<(), String> {
        Self::create_directories(root, &["src", "tests", "crates/katana-ui/tests"])?;
        Self::write(root, "src/editor.rs", "source marker\n")?;
        Self::write(root, "tests/adapter.rs", adapter)?;
        Self::write(
            root,
            "crates/katana-ui/tests/ui_integration_parallel.rs",
            &format!("#[path = \"../../tests/adapter.rs\"]\nmod {registration};\n"),
        )
    }

    pub(super) fn write_kle(root: &Path, selector: &str) -> Result<(), String> {
        Self::create_directories(root, &["tests"])?;
        Self::write(
            root,
            "tests/actual_input.rs",
            &format!("{selector}\n{}", Self::harness_source()),
        )
    }

    fn create_directories(root: &Path, directories: &[&str]) -> Result<(), String> {
        for directory in directories {
            fs::create_dir_all(root.join(directory)).map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    fn write(root: &Path, path: &str, content: &str) -> Result<(), String> {
        fs::write(root.join(path), content).map_err(|error| error.to_string())
    }

    fn harness_source() -> &'static str {
        "fn raw_input(events: Vec<Event>) -> RawInput {\nRawInput {\n}\n}\nimpl StorybookHost {\nfn show_public_api_with_opaque_frame(&mut self) {}\n}\n"
    }
}

#[cfg(test)]
mod tests {
    use super::FixtureBuilder;
    use std::fs;

    #[test]
    fn same_timestamp_fixtures_are_unique_and_cleanup_independently() -> Result<(), String> {
        let first = FixtureBuilder::root_with_timestamp(7)?;
        let second = FixtureBuilder::root_with_timestamp(7)?;
        if first.path() == second.path() {
            return Err("fixtures reused the same path".to_owned());
        }
        let first_path = first.path().to_owned();
        let second_path = second.path().to_owned();
        fs::write(first.path().join("payload"), "first").map_err(|error| error.to_string())?;
        fs::write(second.path().join("payload"), "second").map_err(|error| error.to_string())?;
        if fs::read_to_string(first.path().join("payload")).map_err(|error| error.to_string())?
            != "first"
        {
            return Err("first fixture payload changed".to_owned());
        }
        if fs::read_to_string(second.path().join("payload")).map_err(|error| error.to_string())?
            != "second"
        {
            return Err("second fixture payload changed".to_owned());
        }
        drop(first);
        if first_path.exists() {
            return Err("dropped fixture path still exists".to_owned());
        }
        if fs::read_to_string(second_path.join("payload")).map_err(|error| error.to_string())?
            != "second"
        {
            return Err("dropping first fixture removed second payload".to_owned());
        }
        drop(second);
        Ok(())
    }

    #[test]
    fn failed_existing_path_reservation_preserves_payload() -> Result<(), String> {
        let fixture = FixtureBuilder::root_with_timestamp(11)?;
        let path = fixture.path().to_owned();
        drop(fixture);
        fs::create_dir(&path).map_err(|error| error.to_string())?;
        fs::write(path.join("payload"), "owned").map_err(|error| error.to_string())?;

        let result = FixtureBuilder::reserve_path(path.clone());
        if result.is_ok() {
            return Err("existing fixture path was unexpectedly reserved".to_owned());
        }
        if fs::read_to_string(path.join("payload")).map_err(|error| error.to_string())? != "owned" {
            return Err("failed reservation changed existing payload".to_owned());
        }
        fs::remove_dir_all(path).map_err(|error| error.to_string())?;
        Ok(())
    }
}
