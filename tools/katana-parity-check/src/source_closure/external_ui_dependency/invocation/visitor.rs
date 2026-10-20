use crate::source_closure::ast_resolution::span_to_text;
use crate::source_closure::edge_model::ExternalUiInvocationEdge;
use proc_macro2::Span;
use std::collections::{BTreeMap, BTreeSet};
use syn::Expr;
pub(super) mod bindings;
pub(super) use bindings::context_imports;
mod event;
mod scope;
mod shortcut;
mod text_edit;
mod visits;
#[derive(Default)]
pub(super) struct EdgeVisitor {
    pub(super) file: String,
    pub(super) symbols: Vec<String>,
    pub(super) edges: Vec<ExternalUiInvocationEdge>,
    pub(super) dynamic: bool,
    pub(super) macro_ambiguity: bool,
    pub(super) receiver_ambiguities: Vec<String>,
    pub(super) context_imports: BTreeSet<String>,
    pub(super) context_bindings: BTreeSet<String>,
    pub(super) non_context_bindings: BTreeSet<String>,
    pub(super) text_edit_bindings: BTreeMap<String, usize>,
    pub(super) pending_text_edit_binding: Option<String>,
    pub(super) impl_context: Option<bool>,
    pub(super) function_depth: usize,
    pub(super) closure_depth: usize,
    pub(super) shortcut_closure_depth: usize,
}
impl EdgeVisitor {
    pub(super) fn reset_file(&mut self, path: &str, imports: BTreeSet<String>) {
        self.file = path.into();
        self.symbols.clear();
        self.context_imports = imports;
        self.context_bindings.clear();
        self.non_context_bindings.clear();
        self.text_edit_bindings.clear();
        self.pending_text_edit_binding = None;
        self.closure_depth = 0;
        self.impl_context = None;
        self.function_depth = 0;
        self.shortcut_closure_depth = 0;
        self.receiver_ambiguities.clear();
    }

    pub(super) fn symbol(&self) -> String {
        if self.symbols.is_empty() {
            "crate_file".into()
        } else {
            self.symbols.join("::")
        }
    }

    pub(super) fn add(&mut self, kind: &str, dependency_symbol: &str, span: Span) {
        self.edges.push(ExternalUiInvocationEdge {
            kind: kind.into(),
            source_file: self.file.clone(),
            span: span_to_text(&self.file, &span),
            katana_symbol: self.symbol(),
            dependency_symbol: dependency_symbol.into(),
        });
    }

    pub(super) fn path_symbol(path: &syn::Path) -> String {
        path.segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::")
    }

    pub(super) fn receiver_context(&self, receiver: &Expr) -> Option<bool> {
        let Expr::Path(path) = receiver else {
            return None;
        };
        let symbol = Self::path_symbol(&path.path);
        if symbol == "self" {
            return self.impl_context;
        }
        if path.path.segments.len() != 1 {
            return None;
        }
        let name = symbol.as_str();
        if self.context_bindings.contains(name) {
            Some(true)
        } else if self.non_context_bindings.contains(name) {
            Some(false)
        } else {
            None
        }
    }
}
