use proc_macro2::Span;

use super::super::ast_resolution::span_to_text;
use super::super::scan_state::InputOriginCandidate;
use super::SourceClosureVisitor;

impl<'a> SourceClosureVisitor<'a> {
    pub(crate) fn with_input_candidate<F: FnOnce(&mut Self)>(
        &mut self,
        candidate: InputOriginCandidate,
        f: F,
    ) {
        self.input_candidate_stack.push(candidate);
        f(self);
        self.input_candidate_stack.pop();
    }

    pub(crate) fn with_input_candidates<F: FnOnce(&mut Self)>(
        &mut self,
        candidates: Vec<InputOriginCandidate>,
        f: F,
    ) {
        let added = candidates.len();
        self.input_candidate_stack.extend(candidates);
        f(self);
        for _ in 0..added {
            self.input_candidate_stack.pop();
        }
    }

    pub(crate) fn input_origin_candidates(
        &self,
        construction_span: Span,
    ) -> Vec<InputOriginCandidate> {
        if !self.input_candidate_stack.is_empty() {
            return self.input_candidate_stack.clone();
        }
        if !self.current_symbol_is_host_continuation() {
            return Vec::new();
        }
        vec![InputOriginCandidate {
            kind: "non_ui_host_continuation".to_string(),
            condition_syntax: "non_ui_host_continuation_symbol".to_string(),
            condition_span: span_to_text(self.current_relative_path, &construction_span),
            boundary_syntax: self.current_symbol(),
            boundary_span: span_to_text(self.current_relative_path, &construction_span),
            unresolved_reason: Some(
                "host continuation candidate is not an input origin classification".to_string(),
            ),
        }]
    }

    fn current_symbol_is_host_continuation(&self) -> bool {
        self.symbol_stack.last().is_some_and(|symbol| {
            let symbol = symbol.as_str();
            symbol.contains("continuation")
                || symbol.contains("pending")
                || symbol.contains("confirm")
                || symbol.contains("refresh")
                || symbol.contains("process_")
                || symbol.contains("handle_")
                || symbol.contains("force_")
        })
    }
}
