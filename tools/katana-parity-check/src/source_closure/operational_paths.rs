use std::{
    env,
    path::{Path, PathBuf},
};

use super::operational_cli::{CliOptions, PROFILE_IDS};
use super::operational_input::FIXED_KATANA_REVISION;
use super::operational_process::git_output;

pub(super) fn canonical_dir(value: &str, label: &str) -> Result<PathBuf, String> {
    let path = Path::new(value)
        .canonicalize()
        .map_err(|error| format!("{label} directory is missing or unreadable: {error}"))?;
    if !path.is_dir() {
        return Err(format!(
            "{label} path is not a directory: {}",
            path.display()
        ));
    }
    Ok(path)
}

pub(super) fn canonical_file(value: &str, label: &str) -> Result<PathBuf, String> {
    let path = Path::new(value)
        .canonicalize()
        .map_err(|error| format!("{label} is missing or unreadable: {error}"))?;
    if !path.is_file() {
        return Err(format!("{label} path is not a file: {}", path.display()));
    }
    Ok(path)
}

pub(super) fn capture_run_id(options: &CliOptions) -> Result<String, String> {
    if let Some(values) = options.values.get("run-id")
        && values.len() != 1
    {
        return Err("option --run-id must occur exactly once".into());
    }
    let value = options
        .values
        .get("run-id")
        .and_then(|values| values.first())
        .cloned()
        .or_else(|| env::var("SOURCE_CLOSURE_RUN_ID").ok())
        .or_else(|| {
            let id = env::var("GITHUB_RUN_ID").ok()?;
            let attempt = env::var("GITHUB_RUN_ATTEMPT").unwrap_or_else(|_| "1".into());
            Some(format!("github-{id}-{attempt}"))
        })
        .ok_or_else(|| {
            "capture requires --run-id, SOURCE_CLOSURE_RUN_ID, or GitHub run metadata".to_string()
        })?;
    if value.trim().is_empty()
        || value.contains("::")
        || value.contains('/')
        || value.contains('\\')
    {
        return Err("capture run id contains an invalid path separator or is empty".into());
    }
    Ok(value)
}

pub(super) fn staging_root(output_dir: &str, options: &CliOptions) -> Result<PathBuf, String> {
    super::operational_staging::staging_from_options(output_dir, &capture_run_id(options)?)
}

pub(super) fn validate_profile_id(id: &str) -> Result<(), String> {
    if PROFILE_IDS.contains(&id) {
        Ok(())
    } else {
        Err(format!(
            "profile id must be one of {PROFILE_IDS:?}, got {id}"
        ))
    }
}

pub(super) fn verify_katana_revision(root: &Path) -> Result<(), String> {
    let revision = git_output(root, &["rev-parse", "HEAD"])?;
    if String::from_utf8_lossy(&revision).trim() != FIXED_KATANA_REVISION {
        return Err("KatanA checkout is not the fixed source-closure revision".into());
    }
    Ok(())
}
