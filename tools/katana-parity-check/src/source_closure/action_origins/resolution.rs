use super::super::scan_state::{DispatchArm, FreeFunctionDefinition, MethodDefinition, ScanState};

pub(super) fn resolve_handler_path(
    arm: &DispatchArm,
    state: &ScanState,
    reasons: &mut Vec<String>,
) -> Option<String> {
    if arm.handler_call_facts.len() != 1 {
        return None;
    }
    let call = &arm.handler_call_facts[0];
    if call.kind != "path" {
        return None;
    }
    if arm
        .unresolved_reasons
        .iter()
        .any(|reason| reason.contains("nested") || reason.contains("multiple"))
    {
        reasons.push("handler path call is behind a branch or has multiple calls".to_string());
        return Some(
            "handler_path_resolution=unresolved;reason=branch-or-multiple-call".to_string(),
        );
    }
    let Some(function) = call.syntax.rsplit("::").next() else {
        reasons.push("handler path call has no function segment".to_string());
        return Some("handler_path_resolution=unresolved;reason=empty-function-path".to_string());
    };
    if call
        .path_resolution
        .as_deref()
        .is_some_and(|resolution| resolution.starts_with("unresolved:"))
    {
        let reason = call
            .path_resolution
            .as_deref()
            .unwrap_or("unresolved path resolution");
        reasons.push(format!("handler path call remains unresolved: {reason}"));
        return Some(format!(
            "handler_path_resolution=unresolved;call={};call_span={};reason={reason}",
            call.syntax, call.span
        ));
    }
    let mut candidates = state
        .free_function_definitions
        .iter()
        .filter(|definition| definition.function == function)
        .collect::<Vec<_>>();
    if let Some(target) = call.path_target.as_deref() {
        candidates.retain(|definition| definition.file == target);
    }
    if candidates.len() != 1 {
        let reason = if candidates.is_empty() {
            "no same-source-universe function candidate"
        } else {
            "multiple same-source-universe function candidates"
        };
        reasons.push(format!("handler path call unresolved: {reason}"));
        return Some(format!(
            "handler_path_resolution=unresolved;call={};call_span={};reason={reason}",
            call.syntax, call.span
        ));
    }
    let candidate = candidates[0];
    if free_function_is_recursive(candidate, state, &mut Vec::new()) {
        reasons.push("handler path call unresolved: recursive function route".to_string());
        return Some(format!(
            "handler_path_resolution=unresolved;call={};call_span={};reason=recursive-function-route;definition_file={};definition_symbol={};function_span={};source_span={}",
            call.syntax,
            call.span,
            candidate.file,
            candidate.symbol,
            candidate.function_span,
            candidate.source_span
        ));
    }
    Some(format!(
        "handler_path_resolution=provisional;call={};call_span={};definition_file={};definition_symbol={};function_span={};source_span={}",
        call.syntax,
        call.span,
        candidate.file,
        candidate.symbol,
        candidate.function_span,
        candidate.source_span
    ))
}

fn free_function_is_recursive(
    origin: &FreeFunctionDefinition,
    state: &ScanState,
    visited: &mut Vec<String>,
) -> bool {
    if visited.iter().any(|symbol| symbol == &origin.symbol) {
        return true;
    }
    visited.push(origin.symbol.clone());
    for call in &origin.direct_path_calls {
        let function = call.rsplit("::").next().unwrap_or(call);
        let candidates = state
            .free_function_definitions
            .iter()
            .filter(|definition| definition.function == function)
            .collect::<Vec<_>>();
        if candidates.len() == 1 && free_function_is_recursive(candidates[0], state, visited) {
            return true;
        }
    }
    visited.pop();
    false
}

pub(super) fn resolve_handler_definition<'a>(
    arm: &DispatchArm,
    state: &'a ScanState,
    reasons: &mut Vec<String>,
) -> Option<&'a MethodDefinition> {
    if arm.handler_call_facts.len() != 1 {
        return None;
    }
    let call = &arm.handler_call_facts[0];
    let Some(method) = call.method.as_deref() else {
        reasons.push("handler call is static/path-shaped, not self method syntax".to_string());
        return None;
    };
    if call.kind != "self_method" {
        reasons.push("handler receiver is not self".to_string());
        return None;
    }
    let candidates = state
        .method_definitions
        .iter()
        .filter(|definition| definition.method == method)
        .collect::<Vec<_>>();
    let inherent = candidates
        .iter()
        .filter(|definition| definition.inherent)
        .copied()
        .collect::<Vec<_>>();
    if inherent.is_empty() {
        if candidates.is_empty() {
            reasons.push(format!(
                "missing inherent handler definition for self.{method}"
            ));
        } else {
            reasons.push(format!("trait-only handler definition for self.{method}"));
        }
        return None;
    }
    let Some(receiver_type) = arm.receiver_type.as_deref() else {
        reasons.push(format!(
            "receiver-incompatible handler call self.{method} has no enclosing impl self type"
        ));
        return None;
    };
    let same_type = inherent
        .into_iter()
        .filter(|definition| definition.receiver_type == receiver_type)
        .collect::<Vec<_>>();
    if same_type.is_empty() {
        reasons.push(format!(
            "receiver-incompatible inherent handler definition for self.{method}"
        ));
        return None;
    }
    if same_type
        .iter()
        .any(|definition| definition.receiver_shape == "static")
    {
        reasons.push(format!(
            "static inherent handler definition for self.{method}"
        ));
        return None;
    }
    let compatible = same_type
        .into_iter()
        .filter(|definition| {
            matches!(
                definition.receiver_shape.as_str(),
                "self" | "&self" | "&mut self"
            )
        })
        .collect::<Vec<_>>();
    if compatible.len() != 1 {
        if compatible.is_empty() {
            reasons.push(format!(
                "receiver-incompatible inherent handler definition for self.{method}"
            ));
        } else {
            reasons.push(format!(
                "duplicate inherent handler definitions for self.{method}"
            ));
        }
        return None;
    }
    compatible.first().copied()
}
