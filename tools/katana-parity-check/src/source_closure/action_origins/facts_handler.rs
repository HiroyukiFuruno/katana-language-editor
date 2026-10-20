use super::super::super::scan_state::{
    BranchOutcomeCandidate, DispatchFallthrough, HandlerCallFact, MethodDefinition,
    TerminalEffectCandidate,
};
pub(in crate::source_closure::action_origins) fn branch_outcome_candidate_text(
    candidate: &BranchOutcomeCandidate,
) -> String {
    let branches = if candidate.enclosing_branch_facts.is_empty() {
        "none".to_string()
    } else {
        candidate
            .enclosing_branch_facts
            .iter()
            .map(|fact| format!("{}:{}@{}", fact.kind, fact.syntax, fact.span))
            .collect::<Vec<_>>()
            .join(",")
    };
    let nested = if candidate.enclosing_outcome_syntax.is_empty() {
        "none".to_string()
    } else {
        candidate.enclosing_outcome_syntax.join(",")
    };
    let terminals = if candidate.terminal_effect_candidates.is_empty() {
        "none".to_string()
    } else {
        candidate
            .terminal_effect_candidates
            .iter()
            .map(terminal_effect_candidate_text)
            .collect::<Vec<_>>()
            .join(",")
    };
    format!(
        "branch_outcome_candidate=kind:{};syntax:{};span={};body_span={};body_shape={};enclosing_branches={};nested_outcomes={};terminal_candidates={};resolution=unresolved;unresolved={}",
        candidate.kind,
        candidate.syntax,
        candidate.span,
        candidate.body_span,
        candidate.body_shape,
        branches,
        nested,
        terminals,
        candidate.unresolved.join(",")
    )
}

pub(in crate::source_closure::action_origins) fn handler_body_fact_text(
    fact: &super::super::super::scan_state::HandlerBodyFact,
) -> String {
    format!(
        "handler_body_fact=kind:{};syntax:{};span={}",
        fact.kind, fact.syntax, fact.span
    )
}

pub(in crate::source_closure::action_origins) fn handler_call_fact_text(
    fact: &HandlerCallFact,
) -> String {
    format!(
        "handler_call_fact=syntax:{};kind={};receiver={};method={};span={};path_resolution={};path_target={}",
        fact.syntax,
        fact.kind,
        fact.receiver_shape,
        fact.method.as_deref().unwrap_or("<none>"),
        fact.span,
        fact.path_resolution.as_deref().unwrap_or("none"),
        fact.path_target.as_deref().unwrap_or("none")
    )
}

pub(in crate::source_closure::action_origins) fn terminal_effect_candidate_text(
    candidate: &TerminalEffectCandidate,
) -> String {
    let enclosing = if candidate.enclosing_branch_facts.is_empty() {
        "none".to_string()
    } else {
        candidate
            .enclosing_branch_facts
            .iter()
            .map(|fact| format!("{}:{}@{}", fact.kind, fact.syntax, fact.span))
            .collect::<Vec<_>>()
            .join(",")
    };
    format!(
        "terminal_effect_candidate=kind:{};syntax:{};span={};terminal={};enclosing={}",
        candidate.kind, candidate.syntax, candidate.span, candidate.terminal, enclosing
    )
}

pub(in crate::source_closure::action_origins) fn method_definition_text(
    definition: &MethodDefinition,
) -> String {
    format!(
        "file={};implementation={};method={};receiver={};method_span={};source_span={}",
        definition.file,
        definition.implementation,
        definition.method,
        definition.receiver_shape,
        definition.method_span,
        definition.source_span
    )
}

pub(in crate::source_closure::action_origins) fn dispatch_fallthrough_fact(
    fallthrough: &DispatchFallthrough,
) -> String {
    format!(
        "dispatch_fallthrough_unresolved;file={};dispatch_symbol={};dispatch_span={};arm_span={};reason=fallthrough target is not inferred from helper or pattern syntax",
        fallthrough.file,
        fallthrough.dispatch_symbol,
        fallthrough.dispatch_span,
        fallthrough.arm_span
    )
}
