use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixedSourceHarnessRequest {
    pub katana_source: PathBuf,
    pub kle_source: PathBuf,
    pub kuc_source: PathBuf,
    pub source_closure_profile: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixedSourceHarnessBuilder {
    pub(crate) request: FixedSourceHarnessRequest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixedSourceHarness {
    pub manifest_path: PathBuf,
    pub katana_source: PathBuf,
    pub kle_source: PathBuf,
    pub kuc_source: PathBuf,
    pub source_closure_profile: PathBuf,
    pub katana_revision: String,
}

impl FixedSourceHarness {
    pub fn sandbox_path(&self) -> &Path {
        self.manifest_path.parent().unwrap_or(&self.manifest_path)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixedSourceHarnessMetadata {
    pub metadata_path: PathBuf,
    pub metadata_sha256: String,
    pub cargo_lock_sha256: String,
}
