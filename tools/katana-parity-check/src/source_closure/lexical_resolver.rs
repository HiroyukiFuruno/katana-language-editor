use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use proc_macro2::Span;
use syn::{File, Item};

use super::path_resolution::resolve_rust_name_path_candidates;

mod scope;
use scope::{Scope, scope_from_items};

#[derive(Clone, Debug)]
pub(super) enum LexicalResolution {
    Local(PathBuf),
    Unresolved { reason: String },
    Ambiguous { candidates: Vec<PathBuf> },
    AmbiguousAlias { alias: String },
}

#[derive(Clone, Debug)]
pub(super) struct LexicalPathResolver {
    scopes: Vec<Scope>,
}

impl LexicalPathResolver {
    pub(super) fn from_file(file: &File) -> Self {
        Self {
            scopes: vec![scope_from_items(&file.items)],
        }
    }

    pub(super) fn push_inline_module(&mut self, items: &[Item]) {
        self.scopes.push(scope_from_items(items));
    }

    pub(super) fn pop_inline_module(&mut self) {
        debug_assert!(self.scopes.len() > 1);
        let _ = self.scopes.pop();
    }

    pub(super) fn has_alias(&self, name: &str) -> bool {
        self.scopes
            .iter()
            .rev()
            .any(|scope| scope.aliases.contains_key(name))
    }

    pub(super) fn resolve(
        &self,
        root: &Path,
        current: &Path,
        segments: &[String],
    ) -> LexicalResolution {
        if segments.is_empty() {
            return LexicalResolution::Unresolved {
                reason: "empty Rust path".to_string(),
            };
        }

        self.resolve_expanded(root, current, segments, None, false, None)
    }

    pub(super) fn resolve_with_impl(
        &self,
        root: &Path,
        current: &Path,
        segments: &[String],
        impl_type: Option<&str>,
        inherent_impl: bool,
        inherent_members: Option<&BTreeSet<String>>,
    ) -> LexicalResolution {
        if segments.is_empty() {
            return LexicalResolution::Unresolved {
                reason: "empty Rust path".to_string(),
            };
        }

        self.resolve_expanded(
            root,
            current,
            segments,
            impl_type,
            inherent_impl,
            inherent_members,
        )
    }

    fn resolve_expanded(
        &self,
        root: &Path,
        current: &Path,
        segments: &[String],
        impl_type: Option<&str>,
        inherent_impl: bool,
        inherent_members: Option<&BTreeSet<String>>,
    ) -> LexicalResolution {
        if segments.first().is_some_and(|segment| segment == "Self") {
            return self.resolve_inherent_self(
                current,
                segments,
                impl_type,
                inherent_impl,
                inherent_members,
            );
        }

        let (expanded, alias_span, alias_ambiguous) = self.expand_alias(segments);
        if let Some(alias) = alias_ambiguous {
            return LexicalResolution::AmbiguousAlias { alias };
        }
        let candidates = resolve_rust_name_path_candidates(root, current, &expanded);
        match candidates.as_slice() {
            [candidate] => LexicalResolution::Local(candidate.clone()),
            [] => LexicalResolution::Unresolved {
                reason: match alias_span {
                    Some(_) => format!(
                        "imported path `{}` has no exact local source candidate",
                        segments.join("::")
                    ),
                    None => format!(
                        "path `{}` has no exact local source candidate; external boundaries remain unresolved",
                        segments.join("::")
                    ),
                },
            },
            _ => LexicalResolution::Ambiguous {
                candidates: candidates.to_vec(),
            },
        }
    }

    fn resolve_inherent_self(
        &self,
        current: &Path,
        segments: &[String],
        impl_type: Option<&str>,
        inherent_impl: bool,
        inherent_members: Option<&BTreeSet<String>>,
    ) -> LexicalResolution {
        let method = segments.get(1);
        let defined = impl_type
            .filter(|_| inherent_impl && segments.len() == 2)
            .and(inherent_members)
            .is_some_and(|members| method.is_some_and(|method| members.contains(method)));
        if defined {
            return LexicalResolution::Local(current.to_path_buf());
        }
        LexicalResolution::Unresolved {
            reason: format!(
                "path `{}` is not an exact inherent member in the current impl",
                segments.join("::")
            ),
        }
    }

    fn expand_alias(&self, segments: &[String]) -> (Vec<String>, Option<Span>, Option<String>) {
        let Some(first) = segments.first() else {
            return (Vec::new(), None, None);
        };
        let Some(binding) = self
            .scopes
            .iter()
            .rev()
            .find_map(|scope| scope.aliases.get(first))
        else {
            return (segments.to_vec(), None, None);
        };
        if binding.len() != 1 {
            return (
                segments.to_vec(),
                binding.first().map(|entry| entry.span),
                Some(first.clone()),
            );
        }
        let entry = &binding[0];
        let mut expanded = entry.source.clone();
        expanded.extend(segments.iter().skip(1).cloned());
        (expanded, Some(entry.span), None)
    }
}
