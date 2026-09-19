use super::super::super::scan_state::{ActionConstruction, ActionDefinition, InputOriginCandidate};
pub(in crate::source_closure::action_origins) fn action_id(variant: &str) -> String {
    format!("AppAction::{variant}")
}

pub(in crate::source_closure::action_origins) fn definition_order(
    left: &ActionDefinition,
    right: &ActionDefinition,
) -> std::cmp::Ordering {
    left.file
        .cmp(&right.file)
        .then(left.symbol.cmp(&right.symbol))
        .then(left.variant_span.cmp(&right.variant_span))
}

pub(in crate::source_closure::action_origins) fn construction_order(
    left: &ActionConstruction,
    right: &ActionConstruction,
) -> std::cmp::Ordering {
    left.file
        .cmp(&right.file)
        .then(left.symbol.cmp(&right.symbol))
        .then(left.span.cmp(&right.span))
        .then(left.style.cmp(&right.style))
}

pub(in crate::source_closure::action_origins) fn definition_text(
    definition: &ActionDefinition,
) -> String {
    format!(
        "file={};symbol={};enum_span={};variant_span={}",
        definition.file, definition.symbol, definition.enum_span, definition.variant_span
    )
}

pub(in crate::source_closure::action_origins) fn site_text(
    construction: &ActionConstruction,
) -> String {
    format!(
        "file={};symbol={};span={};action={}::{};style={}",
        construction.file,
        construction.symbol,
        construction.span,
        construction.enum_name,
        construction.variant.as_deref().unwrap_or("<missing>"),
        construction.style
    )
}

pub(in crate::source_closure::action_origins) fn unresolved_site(
    construction: &ActionConstruction,
    reason: &str,
) -> String {
    format!("{};reason={reason}", site_text(construction))
}

pub(in crate::source_closure::action_origins) fn unresolved_input_candidate_facts(
    construction: &ActionConstruction,
    site_reason: &str,
) -> Vec<String> {
    input_candidate_facts(construction).into_iter().map(|fact| format!("input_origin_candidate_index_unresolved;{};site_reason={site_reason};candidate_fact={fact};action origin remains unproven", site_text(construction))).collect()
}

pub(in crate::source_closure::action_origins) fn input_candidate_facts(
    construction: &ActionConstruction,
) -> Vec<String> {
    if construction.input_origin_candidates.is_empty() {
        return vec![format!(
            "input_origin_candidate_unresolved;{};candidate_index=absent;kind=absent_candidate;condition=<absent>;condition_span={};boundary=<absent>;boundary_span={};unresolved=construction outside structurally provable input candidate; action origin remains unproven",
            site_text(construction),
            construction.span,
            construction.span
        )];
    }
    let multiple_reason = (construction.input_origin_candidates.len() > 1)
        .then_some("multiple enclosing input candidates; later phase must prove exactly one route");
    construction
        .input_origin_candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| {
            input_candidate_fact(construction, index, candidate, multiple_reason)
        })
        .collect()
}

fn input_candidate_fact(
    construction: &ActionConstruction,
    index: usize,
    candidate: &InputOriginCandidate,
    multiple_reason: Option<&str>,
) -> String {
    let unresolved = multiple_reason
        .or(candidate.unresolved_reason.as_deref())
        .unwrap_or("none");
    let unresolved = if unresolved == "none" {
        "none".to_string()
    } else {
        format!("{unresolved}; action origin remains unproven")
    };
    format!(
        "input_origin_candidate{};{};candidate_index={index};kind={};condition={};condition_span={};boundary={};boundary_span={};unresolved={}",
        if unresolved == "none" {
            ""
        } else {
            "_unresolved"
        },
        site_text(construction),
        candidate.kind,
        candidate.condition_syntax,
        candidate.condition_span,
        candidate.boundary_syntax,
        candidate.boundary_span,
        unresolved
    )
}

pub(in crate::source_closure::action_origins) fn unresolved_route_fact(
    route: &String,
) -> Option<String> {
    route.contains("terminal_effect_candidate=").then(|| {
        format!(
            "terminal_effect_candidate_index_unresolved;{route}; action origin remains unproven"
        )
    })
}
