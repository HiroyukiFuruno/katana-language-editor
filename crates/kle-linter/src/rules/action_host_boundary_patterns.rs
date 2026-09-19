const ACTION_TYPES: &[&str] = &[
    "EditorAction",
    "EditorActionRequest",
    "EditorActionSource",
    "EditorActionControl",
];

const HOST_TERMS: &[&str] = &[
    "save",
    "format",
    "ingest",
    "diagnostic",
    "problem",
    "gutter",
    "scroll",
    "jump",
    "view",
    "split",
];

pub(super) fn is_action_type(name: &str) -> bool {
    ACTION_TYPES.contains(&name)
}

pub(super) fn is_action_event(owner: &str, member: &str) -> bool {
    owner == "EditorEvent" && member == "ActionRequested"
}

pub(super) fn is_editor_action_path(owner: &str) -> bool {
    owner == "EditorAction"
}

pub(super) fn has_host_term(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    HOST_TERMS.iter().any(|term| lower.contains(term))
}

pub(super) fn is_semantic_construct_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    has_host_term(name)
        && ((lower.starts_with("editor") && lower.contains("action"))
            || lower.contains("request")
            || lower.contains("source")
            || lower.contains("control")
            || lower.contains("payload")
            || lower.contains("command"))
}

pub(super) fn is_action_mapping_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    has_host_term(name)
        && (lower.contains("dispatch")
            || lower.contains("map_")
            || lower.contains("_map")
            || lower.contains("handle_")
            || lower.contains("emit_")
            || lower.contains("forward_"))
}

pub(super) fn is_action_operation_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "request_action"
            | "emit_action"
            | "dispatch_action"
            | "map_action"
            | "forward_action"
            | "handle_action"
            | "action_requested"
    )
}
