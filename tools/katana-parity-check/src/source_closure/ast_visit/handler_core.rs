use syn::visit::{self, Visit};
use syn::{
    Attribute, Block, Expr, ExprAssign, ExprCall, ExprClosure, ExprIf, ExprMatch, ExprMethodCall,
    ExprReturn, Macro, spanned::Spanned,
};

use super::dispatch::is_direct_self_receiver;
use super::syntax::{expr_to_syntax, pat_to_syntax, path_to_syntax};

include!("handler_calls.inc");
include!("handler_branches.inc");

pub(super) fn index_handler_body(
    block: &Block,
    attrs: &[Attribute],
    file: &str,
) -> super::scan_state::HandlerBodyIndex {
    let mut visitor = HandlerBodyVisitor {
        file: file.to_string(),
        facts: Vec::new(),
        branch_outcome_candidates: Vec::new(),
        terminal_effect_candidates: Vec::new(),
        continuation_calls: Vec::new(),
        branch_stack: Vec::new(),
        boundary_stack: Vec::new(),
        unresolved: Vec::new(),
        outcome_stack: Vec::new(),
    };
    for attr in attrs {
        if let Some(predicate) = super::ast_resolution::cfg_predicate(attr) {
            let fact = visitor.make_fact("cfg", predicate, attr.span());
            visitor.facts.push(fact.clone());
            visitor.branch_stack.push(fact);
        }
    }
    visitor.visit_block(block);
    let terminal_count = visitor
        .terminal_effect_candidates
        .iter()
        .filter(|candidate| candidate.terminal)
        .count();
    if terminal_count > 1 {
        visitor
            .unresolved
            .push("multiple terminal-looking return facts".to_string());
        visitor
            .unresolved
            .push("multiple terminal-effect candidate facts".to_string());
    }
    if terminal_count == 0 {
        visitor
            .unresolved
            .push("no terminal-effect candidate facts".to_string());
    }
    (
        visitor.facts,
        visitor.branch_outcome_candidates,
        visitor.terminal_effect_candidates,
        visitor.continuation_calls,
        visitor.unresolved,
    )
}

pub(super) struct HandlerBodyVisitor {
    file: String,
    facts: Vec<super::scan_state::HandlerBodyFact>,
    branch_outcome_candidates: Vec<super::scan_state::BranchOutcomeCandidate>,
    terminal_effect_candidates: Vec<super::scan_state::TerminalEffectCandidate>,
    continuation_calls: Vec<super::scan_state::ContinuationCallFact>,
    branch_stack: Vec<super::scan_state::HandlerBodyFact>,
    boundary_stack: Vec<String>,
    unresolved: Vec<String>,
    outcome_stack: Vec<String>,
}

impl HandlerBodyVisitor {
    pub(super) fn make_fact(
        &self,
        kind: &str,
        syntax: String,
        span: proc_macro2::Span,
    ) -> super::scan_state::HandlerBodyFact {
        super::scan_state::HandlerBodyFact {
            kind: kind.to_string(),
            syntax,
            span: super::ast_resolution::span_to_text(&self.file, &span),
        }
    }

    pub(super) fn fact(&mut self, kind: &str, syntax: String, span: proc_macro2::Span) {
        self.facts.push(self.make_fact(kind, syntax, span));
    }

    pub(super) fn candidate(
        &mut self,
        kind: &str,
        syntax: String,
        span: proc_macro2::Span,
        terminal: bool,
    ) {
        self.terminal_effect_candidates
            .push(super::scan_state::TerminalEffectCandidate {
                kind: kind.to_string(),
                syntax,
                span: super::ast_resolution::span_to_text(&self.file, &span),
                enclosing_branch_facts: self.branch_stack.clone(),
                terminal,
            });
    }

    pub(super) fn record_outcome<F: FnOnce(&mut Self)>(
        &mut self,
        kind: &str,
        syntax: String,
        branch_span: proc_macro2::Span,
        body_span: proc_macro2::Span,
        body_shape: &str,
        f: F,
    ) {
        let terminal_start = self.terminal_effect_candidates.len();
        let unresolved_start = self.unresolved.len();
        let provenance = self.outcome_stack.clone();
        self.outcome_stack.push(syntax.clone());
        f(self);
        self.outcome_stack.pop();
        let terminal_effect_candidates = self.terminal_effect_candidates[terminal_start..]
            .iter()
            .filter(|candidate| candidate.terminal)
            .cloned()
            .collect();
        let mut unresolved = vec![
            "branch outcome semantic effect remains unresolved".to_string(),
            "profile, visibility, availability, action origin, and host effect are not classified"
                .to_string(),
        ];
        unresolved.extend(self.unresolved[unresolved_start..].iter().cloned());
        if !provenance.is_empty() {
            unresolved.push("nested branch provenance remains unresolved".to_string());
        }
        unresolved.sort();
        unresolved.dedup();
        self.branch_outcome_candidates
            .push(super::scan_state::BranchOutcomeCandidate {
                kind: kind.to_string(),
                syntax,
                span: super::ast_resolution::span_to_text(&self.file, &branch_span),
                body_span: super::ast_resolution::span_to_text(&self.file, &body_span),
                body_shape: body_shape.to_string(),
                enclosing_branch_facts: self.branch_stack.clone(),
                enclosing_outcome_syntax: provenance,
                terminal_effect_candidates,
                unresolved,
            });
    }

    pub(super) fn branch_body_shape(&self, block: &Block) -> &'static str {
        if block.stmts.is_empty() {
            "empty"
        } else if block
            .stmts
            .last()
            .is_some_and(|statement| matches!(statement, syn::Stmt::Expr(Expr::Return(_), _)))
        {
            "early_return"
        } else {
            "fallthrough"
        }
    }
}

impl<'ast> Visit<'ast> for HandlerBodyVisitor {
    handler_body_visit_calls!();
    handler_body_visit_branches!();
}
