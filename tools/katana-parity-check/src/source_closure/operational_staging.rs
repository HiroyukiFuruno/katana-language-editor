use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn run_staging_dir(output_dir: &Path, run_id: &str) -> Result<PathBuf, String> {
    if run_id.trim().is_empty()
        || run_id.contains('/')
        || run_id.contains('\\')
        || run_id == "."
        || run_id == ".."
    {
        return Err("run id is not a safe staging directory name".into());
    }
    if output_dir.exists()
        && fs::symlink_metadata(output_dir)
            .map_err(|error| format!("staging parent is unreadable: {error}"))?
            .file_type()
            .is_symlink()
    {
        return Err("staging parent must not be a symlink".into());
    }
    let staging = output_dir.join(run_id);
    if staging.exists()
        && fs::symlink_metadata(&staging)
            .map_err(|error| format!("run staging is unreadable: {error}"))?
            .file_type()
            .is_symlink()
    {
        return Err("run staging must not be a symlink".into());
    }
    Ok(staging)
}

pub(super) fn staging_from_options(output_dir: &str, run_id: &str) -> Result<PathBuf, String> {
    run_staging_dir(Path::new(output_dir), run_id)
}

pub(super) use super::operational_staging_layout::validate_profile_staging;
pub(super) use super::operational_staging_receipt::{
    validate_validation_receipt, write_validation_receipt,
};

pub(super) fn require_staging_run(input_json: &Path) -> Result<(PathBuf, String), String> {
    let input_json = input_json
        .canonicalize()
        .map_err(|error| format!("staging input is missing or unreadable: {error}"))?;
    let assembled = input_json
        .parent()
        .ok_or_else(|| "staging input has no assembled directory".to_string())?;
    if assembled.file_name().and_then(|value| value.to_str()) != Some("assembled")
        || input_json.file_name().and_then(|value| value.to_str())
            != Some("source-closure-input.json")
    {
        return Err("assembled input must be assembled/source-closure-input.json".into());
    }
    let staging = assembled
        .parent()
        .ok_or_else(|| "assembled input has no run staging parent".to_string())?;
    let run_id = staging
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "staging directory has no run id".to_string())?
        .to_string();
    Ok((staging.to_path_buf(), run_id))
}

pub(super) fn staging_root_for_input(input: &Path) -> Result<PathBuf, String> {
    require_staging_run(input).map(|(staging, _)| staging)
}
