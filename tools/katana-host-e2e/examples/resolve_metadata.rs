use std::path::PathBuf;

use katana_host_e2e_fixed::{FixedSourceHarnessBuilder, FixedSourceHarnessRequest};

fn required_path(name: &str) -> Result<PathBuf, String> {
    std::env::var_os(name)
        .map(PathBuf::from)
        .ok_or_else(|| format!("{name} is required"))
}

fn main() -> Result<(), String> {
    let request = FixedSourceHarnessRequest {
        katana_source: required_path("KATANA_SOURCE")?,
        kle_source: required_path("KLE_SOURCE")?,
        kuc_source: required_path("KUC_SOURCE")?,
        source_closure_profile: required_path("SOURCE_CLOSURE_PROFILE")?,
    };
    let harness = FixedSourceHarnessBuilder::new(request)
        .build()
        .map_err(|error| format!("fixed source harness build failed: {error}"))?;
    let metadata = harness
        .resolve_metadata()
        .map_err(|error| format!("fixed source metadata resolution failed: {error}"))?;
    println!(
        "metadata_path={} metadata_sha256={} cargo_lock_sha256={}",
        metadata.metadata_path.display(),
        metadata.metadata_sha256,
        metadata.cargo_lock_sha256
    );
    Ok(())
}
