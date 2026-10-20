use std::collections::BTreeSet;

use syn::visit::Visit;
use syn::{Item, Pat, Type, UseTree};

use super::EdgeVisitor;

pub(super) fn add_context_imports(
    tree: &UseTree,
    prefix: &mut Vec<String>,
    imports: &mut BTreeSet<String>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            add_context_imports(&path.tree, prefix, imports);
            prefix.pop();
        }
        UseTree::Name(name) => {
            prefix.push(name.ident.to_string());
            if prefix == &["egui", "Context"] {
                imports.insert(name.ident.to_string());
            }
            prefix.pop();
        }
        UseTree::Rename(rename) => {
            prefix.push(rename.ident.to_string());
            if prefix == &["egui", "Context"] {
                imports.insert(rename.rename.to_string());
            }
            prefix.pop();
        }
        UseTree::Group(group) => {
            for item in &group.items {
                add_context_imports(item, prefix, imports);
            }
        }
        UseTree::Glob(_) => {}
    }
}

#[derive(Default)]
struct ImportVisitor {
    imports: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for ImportVisitor {
    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        let mut prefix = Vec::new();
        add_context_imports(&node.tree, &mut prefix, &mut self.imports);
        syn::visit::visit_item_use(self, node);
    }
}

pub(in crate::source_closure::external_ui_dependency::invocation) fn context_imports(
    file: &syn::File,
) -> BTreeSet<String> {
    context_imports_for_items(&file.items)
}

pub(super) fn context_imports_for_items(items: &[Item]) -> BTreeSet<String> {
    let mut visitor = ImportVisitor::default();
    for item in items {
        if let Item::Use(item_use) = item {
            visitor.visit_item_use(item_use);
        }
    }
    visitor.imports.retain(|name| {
        !items.iter().any(|item| match item {
            Item::Struct(item) => item.ident != name,
            Item::Enum(item) => item.ident != name,
            Item::Type(item) => item.ident != name,
            Item::Union(item) => item.ident != name,
            _ => false,
        })
    });
    visitor.imports
}

fn binding_name(pat: &Pat) -> Option<String> {
    let Pat::Ident(pat) = pat else {
        return None;
    };
    Some(pat.ident.to_string())
}

fn type_is_context(ty: &Type, imports: &BTreeSet<String>) -> Option<bool> {
    match ty {
        Type::Reference(reference) => type_is_context(&reference.elem, imports),
        Type::Path(path) => {
            let symbol = EdgeVisitor::path_symbol(&path.path);
            if symbol == "egui::Context"
                || (path.path.segments.len() == 1 && imports.contains(&symbol))
            {
                Some(true)
            } else {
                Some(false)
            }
        }
        _ => None,
    }
}

pub(super) fn register_typed_binding(
    pat: &Pat,
    ty: &Type,
    imports: &BTreeSet<String>,
    context_bindings: &mut BTreeSet<String>,
    non_context_bindings: &mut BTreeSet<String>,
) {
    let Some(name) = binding_name(pat) else {
        return;
    };
    match type_is_context(ty, imports) {
        Some(true) => {
            non_context_bindings.remove(&name);
            context_bindings.insert(name);
        }
        Some(false) => {
            context_bindings.remove(&name);
            non_context_bindings.insert(name);
        }
        None => {}
    }
}
