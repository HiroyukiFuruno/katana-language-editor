use std::collections::BTreeSet;
use std::fs;

mod visitor;

#[cfg(test)]
mod tests;

use syn::visit::Visit;

use self::visitor::{EdgeVisitor, context_imports};
use super::super::actual_reference::ActualReferenceBinding;
use super::super::edge_model::ExternalUiInvocationEdge;

type KatanaSource = (String, String);
type KatanaSourceRead = (Vec<KatanaSource>, Vec<String>);

pub(super) fn katana_edges(
    binding: &ActualReferenceBinding<'_>,
) -> (Vec<ExternalUiInvocationEdge>, Vec<String>) {
    let paths = binding
        .input()
        .root
        .evidence
        .katana_tree
        .iter()
        .map(|entry| entry.source_path.clone())
        .collect::<Vec<_>>();
    let (sources, mut unresolved) = read_katana_sources(binding.katana_root(), &paths);
    let (edges, scan_unresolved) = scan_katana_sources(&sources);
    unresolved.extend(scan_unresolved);
    (edges, unresolved)
}

pub(super) fn read_katana_sources(root: &std::path::Path, paths: &[String]) -> KatanaSourceRead {
    let mut sources = Vec::new();
    let mut unresolved = Vec::new();
    for relative in paths {
        let path = root.join(relative);
        match fs::read_to_string(&path) {
            Ok(source) => sources.push((relative.clone(), source)),
            Err(error) => unresolved.push(format!(
                "KatanA invocation source unreadable: {relative}: {error}"
            )),
        }
    }
    (sources, unresolved)
}

pub(super) fn scan_katana_sources(
    sources: &[KatanaSource],
) -> (Vec<ExternalUiInvocationEdge>, Vec<String>) {
    let mut visitor = EdgeVisitor::default();
    let mut unresolved = Vec::new();
    for (path, source) in sources {
        let file = match syn::parse_file(source) {
            Ok(file) => file,
            Err(error) => {
                unresolved.push(format!(
                    "KatanA invocation source AST parse failure: {path}: {error}"
                ));
                continue;
            }
        };
        visitor.reset_file(path, context_imports(&file));
        visitor.visit_file(&file);
        unresolved.extend(visitor.receiver_ambiguities.iter().cloned());
    }
    let found = visitor
        .edges
        .iter()
        .map(|edge| edge.dependency_symbol.as_str())
        .collect::<BTreeSet<_>>();
    for required in super::REQUIRED {
        if !found.contains(required) {
            unresolved.push(format!("missing exact KatanA invocation edge: {required}"));
        }
    }
    if visitor.dynamic || visitor.macro_ambiguity {
        unresolved.push("macro/dynamic ambiguity remains in the bounded invocation scan".into());
    }
    visitor.edges.sort_by(|a, b| {
        (
            a.source_file.as_str(),
            a.span.as_str(),
            a.dependency_symbol.as_str(),
        )
            .cmp(&(
                b.source_file.as_str(),
                b.span.as_str(),
                b.dependency_symbol.as_str(),
            ))
    });
    (visitor.edges, unresolved)
}
