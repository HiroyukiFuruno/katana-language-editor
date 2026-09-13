use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{Attribute, PathSegment};

use super::super::ast_resolution::{cfg_predicate, path_to_symbol};
use super::super::lexical_resolver::LexicalResolution;
use super::super::model::LexicalResolutionStatus;
use super::super::path_resolution::resolve_module_path;
use super::SourceClosureVisitor;

impl<'a> SourceClosureVisitor<'a> {
    pub(crate) fn record_call_target(&mut self, path: &syn::Path, span: Span) {
        let symbol = path_to_symbol(
            &path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>(),
        );
        let segments = path.segments.iter().cloned().collect::<Vec<_>>();
        let resolution = self.resolve_segments(&segments);
        self.record_resolution("call", &symbol, resolution, span);
        self.recorded_call_spans.insert(Self::span_key(span));
    }

    pub(crate) fn record_cfg_attributes(&mut self, attrs: &[Attribute]) {
        for attr in attrs {
            if let Some(predicate) = cfg_predicate(attr) {
                self.record_cfg_edge(&predicate, attr.span());
            }
        }
    }

    pub(crate) fn resolve_module(&self, module: &str) -> Option<std::path::PathBuf> {
        resolve_module_path(self.katana_root, self.current_path, module)
    }

    pub(crate) fn resolve_import(&self, path: &[String]) -> LexicalResolution {
        self.resolve_name_segments(path)
    }

    pub(crate) fn has_import_alias(&self, name: &str) -> bool {
        self.lexical_resolver
            .as_ref()
            .is_some_and(|resolver| resolver.has_alias(name))
    }

    pub(crate) fn record_resolution(
        &mut self,
        kind: &str,
        symbol: &str,
        resolution: LexicalResolution,
        span: Span,
    ) {
        match resolution {
            LexicalResolution::Local(path) => self.record_resolved_edge_with_resolution(
                kind,
                symbol,
                Some(path),
                LexicalResolutionStatus::Local,
                span,
            ),
            LexicalResolution::Unresolved { reason } => self
                .record_unresolved_edge_with_resolution(
                    kind,
                    &format!("{symbol} ({reason})"),
                    Some(LexicalResolutionStatus::Unresolved),
                    span,
                ),
            LexicalResolution::Ambiguous { candidates } => self
                .record_unresolved_edge_with_resolution(
                    kind,
                    &format!(
                        "{symbol} (ambiguous local source candidates: {})",
                        candidates
                            .iter()
                            .map(|candidate| candidate.display().to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    Some(LexicalResolutionStatus::Ambiguous),
                    span,
                ),
            LexicalResolution::AmbiguousAlias { alias } => self
                .record_unresolved_edge_with_resolution(
                    kind,
                    &format!("{symbol} (ambiguous lexical import alias `{alias}`)"),
                    Some(LexicalResolutionStatus::AmbiguousAlias),
                    span,
                ),
        }
    }

    fn resolve_segments(&self, segments: &[PathSegment]) -> LexicalResolution {
        let names = segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        self.resolve_name_segments(&names)
    }

    fn resolve_name_segments(&self, names: &[String]) -> LexicalResolution {
        self.lexical_resolver.as_ref().map_or_else(
            || LexicalResolution::Unresolved {
                reason: "lexical resolver was not initialized".to_string(),
            },
            |resolver| resolver.resolve(self.katana_root, self.current_path, names),
        )
    }
}
