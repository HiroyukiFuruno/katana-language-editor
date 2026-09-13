use std::path::PathBuf;

use proc_macro2::Span;

use super::super::model::LexicalResolutionStatus;
use super::SourceClosureVisitor;

impl<'a> SourceClosureVisitor<'a> {
    pub(crate) fn record_mod_edge(&mut self, to_path: Option<PathBuf>, name: &str, span: Span) {
        let to_path = self.relative_target(to_path);
        self.state.add_edge(
            self.current_relative_path,
            "module",
            &self.current_symbol(),
            to_path.as_deref(),
            Some(name),
            span,
        );
        if let Some(target) = to_path {
            self.discovered_files.push(self.katana_root.join(target));
        }
    }

    pub(crate) fn record_call_edge(&mut self, symbol: &str, path: Option<PathBuf>, span: Span) {
        self.record_resolved_edge("call", symbol, path, span);
    }

    pub(crate) fn record_cfg_edge(&mut self, predicate: &str, span: Span) {
        self.state.add_edge(
            self.current_relative_path,
            "cfg",
            &self.current_symbol(),
            None,
            Some(predicate),
            span,
        );
    }

    pub(crate) fn record_match_edge(&mut self, arm_id: usize, span: Span) {
        self.record_branch_edge("match", Some(&format!("match.arm.{arm_id}")), span);
    }

    pub(crate) fn record_branch_edge(&mut self, kind: &str, label: Option<&str>, span: Span) {
        self.state.add_edge(
            self.current_relative_path,
            kind,
            &self.current_symbol(),
            None,
            label,
            span,
        );
    }

    pub(crate) fn record_unresolved(&mut self, kind: &str, symbol: &str, span: Span) {
        self.record_unresolved_edge(kind, symbol, span);
    }

    pub(crate) fn record_unresolved_edge(&mut self, kind: &str, reason: &str, span: Span) {
        self.record_unresolved_edge_with_resolution(kind, reason, None, span);
    }

    pub(crate) fn record_unresolved_edge_with_resolution(
        &mut self,
        kind: &str,
        reason: &str,
        lexical_resolution: Option<LexicalResolutionStatus>,
        span: Span,
    ) {
        let edge_id = self.state.add_edge(
            self.current_relative_path,
            kind,
            &self.current_symbol(),
            None,
            Some(reason),
            span,
        );
        if let Some(status) = lexical_resolution {
            self.state.set_lexical_resolution(&edge_id, status);
        }
    }

    pub(crate) fn record_resolved_edge_with_resolution(
        &mut self,
        kind: &str,
        symbol: &str,
        path: Option<PathBuf>,
        lexical_resolution: LexicalResolutionStatus,
        span: Span,
    ) {
        let to_path = self.relative_target(path);
        let edge_id = self.state.add_edge(
            self.current_relative_path,
            kind,
            &self.current_symbol(),
            to_path.as_deref(),
            Some(symbol),
            span,
        );
        self.state
            .set_lexical_resolution(&edge_id, lexical_resolution);
        if let Some(target) = to_path {
            self.discovered_files.push(self.katana_root.join(target));
        }
    }

    pub(crate) fn record_resolved_edge(
        &mut self,
        kind: &str,
        symbol: &str,
        path: Option<PathBuf>,
        span: Span,
    ) {
        let to_path = self.relative_target(path);
        self.state.add_edge(
            self.current_relative_path,
            kind,
            &self.current_symbol(),
            to_path.as_deref(),
            Some(symbol),
            span,
        );
        if let Some(target) = to_path {
            self.discovered_files.push(self.katana_root.join(target));
        }
    }

    fn relative_target(&self, path: Option<PathBuf>) -> Option<String> {
        path.map(|path| {
            let path = path.strip_prefix(self.katana_root).unwrap_or(&path);
            path.to_string_lossy()
                .replace(std::path::MAIN_SEPARATOR, "/")
        })
    }
}
