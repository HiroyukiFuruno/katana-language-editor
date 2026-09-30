mod invocation;
mod lock;
mod output;
mod source;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod archive_layout_tests;

use std::fs;

use super::actual_reference::ActualReferenceBinding;
use super::edge_model::ExternalUiDependency;
use invocation::{katana_edges, scan_katana_sources};
use lock::{LockPackage, parse_lock, resolve_package_archive, resolve_package_root};
use output::finish;
use source::dependency_sources;

#[cfg(test)]
use invocation::read_katana_sources;
#[cfg(test)]
use source::DependencyDefinitionVisitor;

pub(super) const PACKAGE: &str = "egui";
pub(super) const REQUIRED: [&str; 6] = [
    "TextEdit::multiline",
    "TextEdit::load_state",
    "TextEdit::store_state",
    "TextEdit::show",
    "Event::Paste",
    "InputState::consume_shortcut",
];

pub(super) fn materialize(binding: &ActualReferenceBinding<'_>) -> ExternalUiDependency {
    let mut unresolved = Vec::new();
    let lock = match fs::read(binding.katana_root().join("Cargo.lock")) {
        Ok(bytes) => bytes,
        Err(error) => {
            unresolved.push(format!("missing fixed-reference Cargo.lock: {error}"));
            return finish(
                LockPackage::default(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                unresolved,
            );
        }
    };
    let Some(lock_package) = parse_lock(&lock, &mut unresolved) else {
        return finish(
            LockPackage::default(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            unresolved,
        );
    };
    let Some(package_root) =
        resolve_package_root(binding.katana_root(), &lock_package, &mut unresolved)
    else {
        return finish(
            lock_package,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            unresolved,
        );
    };
    let Some(archive) = resolve_package_archive(&package_root, &lock_package, &mut unresolved)
    else {
        return finish(
            lock_package,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            unresolved,
        );
    };
    let (source_files, source_symbols, symbol_spans, direct_semantic_edges, source_unresolved) =
        dependency_sources(&package_root, &archive, &lock_package);
    unresolved.extend(source_unresolved);
    let (edges, edge_unresolved) = katana_edges(binding);
    unresolved.extend(edge_unresolved);
    finish(
        lock_package,
        source_files,
        source_symbols,
        symbol_spans,
        direct_semantic_edges,
        edges,
        unresolved,
    )
}

pub(super) fn audit_locked_sources(
    reference_root: &std::path::Path,
    fixed_lock_bytes: &[u8],
    sources: &[(String, String)],
) -> ExternalUiDependency {
    let (edges, mut unresolved) = scan_katana_sources(sources);
    unresolved.push("diagnostic does not capture transitive semantic closure".into());
    let Some(lock_package) = parse_lock(fixed_lock_bytes, &mut unresolved) else {
        return finish(
            LockPackage::default(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            edges,
            unresolved,
        );
    };
    let Some(package_root) = resolve_package_root(reference_root, &lock_package, &mut unresolved)
    else {
        return finish(
            lock_package,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            edges,
            unresolved,
        );
    };
    let Some(archive) = resolve_package_archive(&package_root, &lock_package, &mut unresolved)
    else {
        return finish(
            lock_package,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            edges,
            unresolved,
        );
    };
    let (files, symbols, spans, direct_semantic_edges, source_unresolved) =
        dependency_sources(&package_root, &archive, &lock_package);
    unresolved.extend(source_unresolved);
    finish(
        lock_package,
        files,
        symbols,
        spans,
        direct_semantic_edges,
        edges,
        unresolved,
    )
}
