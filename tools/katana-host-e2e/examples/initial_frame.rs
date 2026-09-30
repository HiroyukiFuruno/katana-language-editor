#[cfg(not(feature = "fixed-host"))]
fn main() {
    eprintln!("initial_frame requires --features fixed-host");
    std::process::exit(2);
}

#[cfg(feature = "fixed-host")]
fn main() -> Result<(), String> {
    use std::path::PathBuf;

    use katana_host_e2e_fixed::{
        FixedHostSession, FixedSourceHarnessBuilder, FixedSourceHarnessRequest,
    };

    fn required_path(name: &str) -> Result<PathBuf, String> {
        std::env::var_os(name)
            .map(PathBuf::from)
            .ok_or_else(|| format!("{name} is required"))
    }

    let harness = FixedSourceHarnessBuilder::new(FixedSourceHarnessRequest {
        katana_source: required_path("KATANA_SOURCE")?,
        kle_source: required_path("KLE_SOURCE")?,
        kuc_source: required_path("KUC_SOURCE")?,
        source_closure_profile: required_path("SOURCE_CLOSURE_PROFILE")?,
    })
    .build()
    .map_err(|error| format!("fixed source harness build failed: {error}"))?;
    let metadata = harness
        .resolve_metadata()
        .map_err(|error| format!("fixed source metadata resolution failed: {error}"))?;
    let mut session = FixedHostSession::new(harness.sandbox_path());
    let frame = session
        .initial_frame()
        .map_err(|error| format!("fixed host initial frame failed: {error}"))?;

    println!("metadata_path={}", metadata.metadata_path.display());
    println!("metadata_sha256={}", metadata.metadata_sha256);
    println!("cargo_lock_sha256={}", metadata.cargo_lock_sha256);
    println!("frame_sha256={}", hex_digest(&frame.frame_hash));
    println!(
        "accesskit_root_sha256={}",
        hex_digest(&frame.accesskit.root_hash)
    );
    Ok(())
}

#[cfg(feature = "fixed-host")]
const SHA256_DIGEST_BYTES: usize = 32;

#[cfg(feature = "fixed-host")]
fn hex_digest(bytes: &[u8; SHA256_DIGEST_BYTES]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
