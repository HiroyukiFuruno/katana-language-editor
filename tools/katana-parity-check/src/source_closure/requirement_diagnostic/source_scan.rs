use std::collections::{BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

use syn::parse_file;

use super::super::ast_scan::SourceClosureVisitor;
use super::super::fingerprint::sha256_hex;
use super::super::scan_state::ScanState;
use super::checkout::{checked_path, read_and_verify};

pub(super) fn scan_source(
    root: &Path,
    path: &Path,
    state: &mut ScanState,
    queue: &mut VecDeque<PathBuf>,
    seen: &mut BTreeSet<PathBuf>,
) -> Result<Option<(String, String)>, String> {
    let canonical = checked_path(root, path)?;
    if !seen.insert(canonical.clone()) {
        return Ok(None);
    }
    let relative = canonical
        .strip_prefix(root)
        .map_err(|_| "source path escapes KatanA checkout".to_string())?
        .to_string_lossy()
        .replace('\\', "/");
    let bytes = read_and_verify(root, &relative)?;
    let hash = sha256_hex(&bytes);
    let text = std::str::from_utf8(&bytes)
        .map_err(|error| format!("source is not UTF-8: {relative}: {error}"))?;
    let syntax =
        parse_file(text).map_err(|error| format!("failed to parse source {relative}: {error}"))?;
    state.record_file(relative.clone(), hash);
    let mut discovered = Vec::new();
    SourceClosureVisitor::new(root, &canonical, &relative, state, &mut discovered)
        .scan_file(&syntax);
    for child in discovered {
        match checked_path(root, &child) {
            Ok(child) => queue.push_back(child),
            Err(error) => {
                let io_error = std::io::Error::other(error);
                let child_relative = child
                    .strip_prefix(root)
                    .map(|path| path.to_string_lossy().replace('\\', "/"))
                    .unwrap_or_else(|_| child.to_string_lossy().into_owned());
                state.mark_target_unresolved(&child_relative, &io_error);
            }
        }
    }
    let _ = read_and_verify(root, &relative)?;
    Ok(Some((relative, text.to_owned())))
}
