use std::path::{Path, PathBuf};

pub(super) fn resolve_seed_path(root: &Path, seed: &str) -> Result<PathBuf, String> {
    if seed.trim().is_empty() || Path::new(seed).is_absolute() || seed.contains('\\') {
        return Err(format!("seed path is not canonical relative: {seed}"));
    }
    let candidate = root.join(seed);
    if !candidate.exists() {
        return Err(format!("seed path does not exist: {seed}"));
    }
    if candidate.is_dir() {
        return Err(format!("seed path must be a file: {seed}"));
    }
    Ok(candidate)
}
