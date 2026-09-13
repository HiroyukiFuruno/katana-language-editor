use std::collections::{BTreeSet, VecDeque};
use std::fs;
use std::path::PathBuf;

use syn::parse_file;

use super::super::actual_reference::ActualReferenceBinding;
use super::super::ast_scan::SourceClosureVisitor;
use super::super::fingerprint::sha256_hex;
use super::super::path_resolution::relative_path;
use super::super::scan_state::ScanState;

pub(super) fn source(
    binding: &ActualReferenceBinding<'_>,
    source_path: PathBuf,
    state: &mut ScanState,
    discovered: &mut VecDeque<PathBuf>,
    seen: &mut BTreeSet<PathBuf>,
) -> Result<(), String> {
    if !seen.insert(source_path.clone()) {
        return Ok(());
    }
    let source_bytes = fs::read(&source_path).map_err(|error| {
        format!(
            "failed to read seed source {}: {error}",
            source_path.display()
        )
    })?;
    let source_hash = sha256_hex(&source_bytes);
    let source = std::str::from_utf8(&source_bytes).map_err(|error| {
        format!(
            "seed source {} is not valid UTF-8: {error}",
            source_path.display()
        )
    })?;
    let file = parse_file(source).map_err(|error| {
        format!(
            "failed to parse seed source {}: {error}",
            source_path.display()
        )
    })?;
    let current_relative_path = relative_path(binding.katana_root(), &source_path);
    binding.verify_scanned_file(&current_relative_path, &source_hash)?;
    state.record_file(current_relative_path.clone(), source_hash);
    let mut discovered_children = Vec::new();
    SourceClosureVisitor::new(
        binding.katana_root(),
        &source_path,
        &current_relative_path,
        state,
        &mut discovered_children,
    )
    .scan_file(&file);
    for child in discovered_children {
        match child.canonicalize() {
            Ok(child_path) => discovered.push_back(child_path),
            Err(error) => {
                state.mark_target_unresolved(&relative_path(binding.katana_root(), &child), &error)
            }
        }
    }
    Ok(())
}
