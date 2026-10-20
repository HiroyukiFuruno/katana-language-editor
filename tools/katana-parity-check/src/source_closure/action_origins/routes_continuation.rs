use super::super::super::scan_state::{MethodDefinition, ScanState};
pub(in crate::source_closure::action_origins) fn continuation_route_facts(
    source: &MethodDefinition,
    state: &ScanState,
) -> Vec<String> {
    let mut calls = source.continuation_calls.clone();
    calls.sort_by(|left, right| {
        left.span
            .cmp(&right.span)
            .then(left.syntax.cmp(&right.syntax))
            .then(left.method.cmp(&right.method))
    });
    calls.into_iter().map(|call| {
         let candidates = state.method_definitions.iter().filter(|definition| definition.inherent && definition.implementation == source.implementation && definition.receiver_type == source.receiver_type && definition.method == call.method && matches!(definition.receiver_shape.as_str(), "self" | "&self" | "&mut self")).collect::<Vec<_>>();
         let mut reasons = Vec::new();
         let status = continuation_status(source, &call, &candidates, state, &mut reasons);
         let target_definition = candidates.first().map_or_else(|| "<unresolved>".to_string(), |definition| definition.source_span.clone());
         format!("same_implementation_continuation_route;status={status};call_syntax={};call_span={};source_definition_span={};target_definition_span={};implementation={};target_method={};unresolved={}", call.syntax, call.span, source.source_span, target_definition, source.implementation, call.method, if reasons.is_empty() { "none".to_string() } else { reasons.join("|") })
     }).collect()
}

fn continuation_status(
    source: &MethodDefinition,
    call: &super::super::super::scan_state::ContinuationCallFact,
    candidates: &[&MethodDefinition],
    state: &ScanState,
    reasons: &mut Vec<String>,
) -> &'static str {
    if !call.enclosing_branch_facts.is_empty() {
        reasons.push("branch-contained continuation".to_string());
        return "unresolved";
    }
    if !call.boundary_kinds.is_empty() {
        reasons.push(format!("{} boundary", call.boundary_kinds.join("/")));
        return "unresolved";
    }
    if candidates.is_empty() {
        reasons.push(format!(
            "zero receiver-compatible inherent candidates for self.{}",
            call.method
        ));
        return "unresolved";
    }
    if candidates.len() > 1 {
        reasons.push(format!(
            "multiple receiver-compatible inherent candidates for self.{}",
            call.method
        ));
        return "unresolved";
    }
    let target = candidates[0];
    if source.method == target.method && source.source_span == target.source_span {
        reasons.push("direct recursion".to_string());
        return "unresolved";
    }
    if continuation_cycle(source, target, state, &mut Vec::new()) {
        reasons.push("mutually recursive continuation loop".to_string());
        return "unresolved";
    }
    if !target
        .terminal_effect_candidates
        .iter()
        .any(|candidate| candidate.terminal)
    {
        reasons.push("no terminal candidate".to_string());
        return "unresolved";
    }
    "provisional"
}

fn continuation_cycle(
    origin: &MethodDefinition,
    current: &MethodDefinition,
    state: &ScanState,
    visited: &mut Vec<String>,
) -> bool {
    let key = current.source_span.clone();
    if key == origin.source_span {
        return true;
    }
    if visited.iter().any(|seen| seen == &key) {
        return false;
    }
    visited.push(key);
    for call in &current.continuation_calls {
        if !call.enclosing_branch_facts.is_empty() || !call.boundary_kinds.is_empty() {
            continue;
        }
        let candidates = state
            .method_definitions
            .iter()
            .filter(|definition| {
                definition.inherent
                    && definition.implementation == current.implementation
                    && definition.receiver_type == current.receiver_type
                    && definition.method == call.method
                    && matches!(
                        definition.receiver_shape.as_str(),
                        "self" | "&self" | "&mut self"
                    )
            })
            .collect::<Vec<_>>();
        if candidates.len() == 1 && continuation_cycle(origin, candidates[0], state, visited) {
            return true;
        }
    }
    false
}
