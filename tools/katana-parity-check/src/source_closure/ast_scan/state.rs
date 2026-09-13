use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use proc_macro2::Span;
use syn::File;
use syn::visit::Visit;

use super::super::lexical_resolver::LexicalPathResolver;
use super::super::scan_state::ScanState;

pub(crate) struct SourceClosureVisitor<'a> {
    pub(crate) katana_root: &'a Path,
    pub(crate) current_path: &'a Path,
    pub(crate) current_relative_path: &'a str,
    pub(crate) state: &'a mut ScanState,
    pub(crate) discovered_files: &'a mut Vec<PathBuf>,
    pub(crate) symbol_stack: Vec<String>,
    pub(crate) impl_type_stack: Vec<String>,
    pub(crate) lexical_resolver: Option<LexicalPathResolver>,
    pub(crate) recorded_call_spans: BTreeSet<String>,
    pub(crate) input_candidate_stack: Vec<super::super::scan_state::InputOriginCandidate>,
    pub(crate) pattern_depth: usize,
    pub(crate) dispatch_match_depth: usize,
}

impl<'a> SourceClosureVisitor<'a> {
    pub(crate) fn is_dispatch_symbol(symbol: &str) -> bool {
        matches!(
            symbol.rsplit("::").next(),
            Some("dispatch_action" | "dispatch_secondary" | "dispatch_tertiary")
        )
    }

    pub(crate) fn new(
        katana_root: &'a Path,
        current_path: &'a Path,
        current_relative_path: &'a str,
        state: &'a mut ScanState,
        discovered_files: &'a mut Vec<PathBuf>,
    ) -> Self {
        Self {
            katana_root,
            current_path,
            current_relative_path,
            state,
            discovered_files,
            symbol_stack: Vec::new(),
            impl_type_stack: Vec::new(),
            lexical_resolver: None,
            recorded_call_spans: BTreeSet::new(),
            input_candidate_stack: Vec::new(),
            pattern_depth: 0,
            dispatch_match_depth: 0,
        }
    }

    pub(crate) fn scan_file<'ast>(&mut self, file: &'ast File)
    where
        Self: Visit<'ast>,
    {
        self.lexical_resolver = Some(LexicalPathResolver::from_file(file));
        self.visit_file(file);
    }

    pub(crate) fn current_symbol(&self) -> String {
        if self.symbol_stack.is_empty() {
            "crate_file".to_string()
        } else {
            self.symbol_stack.join("::")
        }
    }

    pub(crate) fn with_symbol<F: FnOnce(&mut Self)>(&mut self, symbol: String, f: F) {
        self.symbol_stack.push(symbol);
        f(self);
        self.symbol_stack.pop();
    }

    pub(crate) fn current_impl_type(&self) -> Option<String> {
        self.impl_type_stack.last().cloned()
    }

    pub(crate) fn with_impl_type<F: FnOnce(&mut Self)>(&mut self, type_name: String, f: F) {
        self.impl_type_stack.push(type_name);
        f(self);
        self.impl_type_stack.pop();
    }

    pub(crate) fn with_pattern<F: FnOnce(&mut Self)>(&mut self, f: F) {
        self.pattern_depth += 1;
        f(self);
        self.pattern_depth -= 1;
    }

    pub(crate) fn in_pattern(&self) -> bool {
        self.pattern_depth > 0
    }

    pub(crate) fn call_was_recorded(&self, span: Span) -> bool {
        self.recorded_call_spans.contains(&Self::span_key(span))
    }

    pub(crate) fn push_inline_scope(&mut self, items: &[syn::Item]) {
        if let Some(resolver) = self.lexical_resolver.as_mut() {
            resolver.push_inline_module(items);
        }
    }

    pub(crate) fn pop_inline_scope(&mut self) {
        if let Some(resolver) = self.lexical_resolver.as_mut() {
            resolver.pop_inline_module();
        }
    }

    pub(crate) fn span_key(span: Span) -> String {
        let start = span.start();
        let end = span.end();
        format!(
            "{}:{}-{}:{}",
            start.line, start.column, end.line, end.column
        )
    }
}
