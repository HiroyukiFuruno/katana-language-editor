use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use syn::visit::Visit;

use super::super::edge_model::{
    ExternalUiDirectSemanticEdge, ExternalUiSymbolSpan, SourceFileHash,
};
use super::super::fingerprint::sha256_hex;
use super::REQUIRED;
use super::lock::LockPackage;

mod archive;
#[cfg(test)]
mod archive_tests;
mod checksum;
mod definition;
#[cfg(test)]
mod inventory_archive_tests;
#[cfg(test)]
mod inventory_tests;
mod path_validation;
#[cfg(test)]
mod path_validation_tests;
#[cfg(test)]
mod registry_source_tests;
mod semantic;
mod source_tree;
pub(super) use definition::DependencyDefinitionVisitor;
use definition::add_required_span;

type DependencySourceEvidence = (
    Vec<SourceFileHash>,
    Vec<String>,
    Vec<ExternalUiSymbolSpan>,
    Vec<ExternalUiDirectSemanticEdge>,
    Vec<String>,
);

type ParsedSourceEvidence = (
    Vec<SourceFileHash>,
    BTreeSet<String>,
    Vec<ExternalUiSymbolSpan>,
    Vec<ExternalUiDirectSemanticEdge>,
);

pub(super) fn dependency_sources(
    root: &Path,
    archive_path: &Path,
    lock: &LockPackage,
) -> DependencySourceEvidence {
    let mut unresolved = Vec::new();
    let checksum = match checksum::optional_file_hashes(root, &lock.checksum) {
        Ok(checksum) => checksum,
        Err(error) => {
            unresolved.push(error);
            return (Vec::new(), Vec::new(), Vec::new(), Vec::new(), unresolved);
        }
    };
    let archive_files = match archive::file_hashes_from_archive(archive_path, lock) {
        Ok(hashes) => hashes,
        Err(error) => {
            unresolved.push(error);
            return (Vec::new(), Vec::new(), Vec::new(), Vec::new(), unresolved);
        }
    };
    if let Some(manifest_hashes) = &checksum
        && manifest_hashes != &archive_files
    {
        unresolved.push("egui registry archive files differ from checksum manifest".into());
    }
    let archive_hashes = archive_files
        .into_iter()
        .filter(|(path, _)| path.ends_with(".rs"))
        .collect::<BTreeMap<_, _>>();
    match source_tree::rust_paths(root) {
        Ok(paths) if paths == archive_hashes.keys().cloned().collect() => {}
        Ok(_) => unresolved.push("egui source Rust paths differ from registry archive".into()),
        Err(error) => unresolved.push(error),
    }
    if !unresolved.is_empty() {
        return (Vec::new(), Vec::new(), Vec::new(), Vec::new(), unresolved);
    }
    let sources = source_bytes(root, &archive_hashes, checksum.as_ref(), &mut unresolved);
    if !unresolved.is_empty() {
        return (Vec::new(), Vec::new(), Vec::new(), Vec::new(), unresolved);
    }
    let (files, symbols, symbol_spans, direct_semantic_edges) =
        parse_sources(sources, &mut unresolved);
    for required in REQUIRED {
        if !symbols.contains(required) {
            unresolved.push(format!("missing exact egui definition: {required}"));
        }
    }
    (
        files,
        symbols.into_iter().collect(),
        symbol_spans,
        direct_semantic_edges,
        unresolved,
    )
}

fn source_bytes(
    root: &Path,
    archive_hashes: &BTreeMap<String, String>,
    checksum_hashes: Option<&BTreeMap<String, String>>,
    unresolved: &mut Vec<String>,
) -> BTreeMap<String, Vec<u8>> {
    let mut sources = BTreeMap::new();
    for (relative, archive_hash) in archive_hashes {
        let path = match path_validation::source_path(root, relative) {
            Ok(path) => path,
            Err(error) => {
                unresolved.push(error);
                continue;
            }
        };
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) => {
                unresolved.push(format!("egui source file unreadable: {relative}: {error}"));
                continue;
            }
        };
        let actual_hash = sha256_hex(&bytes);
        let actual = actual_hash.trim_start_matches("sha256:");
        if actual != archive_hash
            || checksum_hashes
                .and_then(|hashes| hashes.get(relative))
                .is_some_and(|checksum_hash| checksum_hash != actual)
        {
            unresolved.push(format!("egui source/hash mismatch: {relative}"));
            continue;
        }
        sources.insert(relative.clone(), bytes);
    }
    sources
}

fn parse_sources(
    sources: BTreeMap<String, Vec<u8>>,
    unresolved: &mut Vec<String>,
) -> ParsedSourceEvidence {
    let mut files = Vec::new();
    let mut symbols = BTreeSet::new();
    let mut symbol_spans = Vec::new();
    let mut parsed = Vec::new();
    for (relative, bytes) in sources {
        let source = match std::str::from_utf8(&bytes) {
            Ok(source) => source,
            Err(error) => {
                unresolved.push(format!(
                    "egui source file is not UTF-8: {relative}: {error}"
                ));
                continue;
            }
        };
        let file = match syn::parse_file(source) {
            Ok(file) => file,
            Err(error) => {
                unresolved.push(format!(
                    "egui source AST parse failure: {relative}: {error}"
                ));
                continue;
            }
        };
        files.push(SourceFileHash {
            path: format!("egui/{relative}"),
            sha256: sha256_hex(&bytes),
        });
        let mut visitor = DependencyDefinitionVisitor::default();
        visitor.visit_file(&file);
        let found = visitor
            .symbols
            .iter()
            .filter(|symbol| REQUIRED.contains(&symbol.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        if !found.is_empty() {
            symbols.extend(found);
            for symbol in &visitor.symbols {
                add_required_span(&visitor, symbol, &relative, &mut symbol_spans);
            }
        }
        parsed.push((relative, file));
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    symbol_spans.sort_by(|a, b| (&a.symbol, &a.span).cmp(&(&b.symbol, &b.span)));
    let direct_semantic_edges = semantic::collect(&parsed);
    (files, symbols, symbol_spans, direct_semantic_edges)
}
