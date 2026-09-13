use super::super::scan_state::{DispatchArm, ScanState};
use super::{facts, resolution};

#[path = "routes_continuation.rs"]
mod continuation;
use continuation::continuation_route_facts;
pub(super) fn dispatch_route_fact(arm: &DispatchArm, state: &ScanState) -> String {
    let handlers = if arm.handler_calls.is_empty() {
        "<none>".to_string()
    } else {
        arm.handler_calls.join("|")
    };
    let mut reasons = arm.unresolved_reasons.clone();
    let handler_definition = resolution::resolve_handler_definition(arm, state, &mut reasons);
    let path_resolution = resolution::resolve_handler_path(arm, state, &mut reasons);
    let handler_fact = handler_definition.as_ref().map_or_else(
        || "handler_definition=unresolved".to_string(),
        |definition| {
            format!(
                "handler_definition={}",
                facts::method_definition_text(definition)
            )
        },
    );
    let body_fact = handler_definition.as_ref().map_or_else(
        || "handler_body_fact=<unresolved-handler-definition>".to_string(),
        |definition| {
            reasons.push("handler body final effect remains unclassified".to_string());
            reasons.extend(definition.body_unresolved.iter().cloned());
            if definition.body_facts.is_empty() {
                "handler_body_fact=<none>".to_string()
            } else {
                definition
                    .body_facts
                    .iter()
                    .map(facts::handler_body_fact_text)
                    .collect::<Vec<_>>()
                    .join("|")
            }
        },
    );
    let terminal_candidate_fact = handler_definition.as_ref().map_or_else(
        || "terminal_effect_candidate=<unresolved-handler-definition>".to_string(),
        |definition| {
            if definition.terminal_effect_candidates.is_empty() {
                "terminal_effect_candidate=none;unresolved=no terminal-effect candidate facts"
                    .to_string()
            } else {
                definition
                    .terminal_effect_candidates
                    .iter()
                    .map(facts::terminal_effect_candidate_text)
                    .collect::<Vec<_>>()
                    .join("|")
            }
        },
    );
    let branch_outcome_fact = handler_definition.as_ref().map_or_else(|| "branch_outcome_candidate=<unresolved-handler-definition>".to_string(), |definition| { if definition.branch_outcome_candidates.is_empty() { "branch_outcome_candidate=none;unresolved=handler body has no indexed branch outcomes".to_string() } else { definition.branch_outcome_candidates.iter().map(facts::branch_outcome_candidate_text).collect::<Vec<_>>().join("|") } });
    let continuation_route_facts = handler_definition
        .as_ref()
        .map_or_else(Vec::new, |definition| {
            continuation_route_facts(definition, state)
        });
    reasons.extend(
        continuation_route_facts
            .iter()
            .filter(|fact| !fact.contains("status=provisional"))
            .cloned(),
    );
    let continuation_routes = if continuation_route_facts.is_empty() {
        "continuation_route=<none>".to_string()
    } else {
        continuation_route_facts.join("|")
    };
    let call_fact = arm.handler_call_facts.first().map_or_else(
        || "handler_call_fact=<none>".to_string(),
        facts::handler_call_fact_text,
    );
    let path_fact = path_resolution
        .unwrap_or_else(|| "handler_path_resolution=unresolved;candidate=<none>".to_string());
    format!(
        "provisional_forward_route;file={};dispatch_symbol={};dispatch_span={};action=AppAction::{};variant_pattern_span={};arm_span={};handlers={};{};{};{};{};{};{};{};unresolved={}",
        arm.file,
        arm.dispatch_symbol,
        arm.dispatch_span,
        arm.action_variant,
        arm.variant_pattern_span,
        arm.arm_span,
        handlers,
        handler_fact,
        call_fact,
        path_fact,
        body_fact,
        terminal_candidate_fact,
        branch_outcome_fact,
        continuation_routes,
        if reasons.is_empty() {
            "none".to_string()
        } else {
            reasons.join("|")
        }
    )
}
