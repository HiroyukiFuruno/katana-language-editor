const FORBIDDEN_TYPES: &[&str] = &[
    "EditorAuthoringCommand",
    "EditorCodeBlockKind",
    "EditorAuthoringMenuState",
    "EditorAuthoringControl",
    "RunAuthoringCommand",
    "MarkdownAuthoringOp",
    "CodeBlockKind",
];

const STATE_WORDS: &[&str] = &["cursor", "selection", "range", "anchor"];

pub(super) fn is_forbidden_type(name: &str) -> bool {
    FORBIDDEN_TYPES.contains(&name)
}

pub(super) fn is_authoring_context(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    name.contains("authoring")
        || name.contains("markdown_toolbar")
        || name.contains("code_block_menu")
}

pub(super) fn is_authoring_state_name(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    if name == "authoring_menu" {
        return false;
    }
    name.contains("authoring") && STATE_WORDS.iter().any(|word| name.contains(word))
        || (name.contains("authoring")
            && (name.contains("popup") || name.contains("menu") || name.contains("lifecycle")))
}

pub(super) fn is_state_name(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    STATE_WORDS.iter().any(|word| name.contains(word))
}

pub(super) fn is_authoring_mapping_name(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    let mapping = [
        "map",
        "mapping",
        "to_action",
        "to_operation",
        "from_command",
        "transform",
        "apply",
        "run",
    ];
    (name.contains("authoring") && mapping.iter().any(|word| name.contains(word)))
        || name == "author_markdown"
        || (name.contains("markdown") && (name.contains("transform") || name.contains("operation")))
        || (name.contains("command") && name.contains("operation"))
        || (name.contains("command") && (name.contains("action") || name.contains("markdown")))
        || (name.contains("code_block")
            && (name.contains("operation") || name.contains("action") || name.contains("map")))
}

pub(super) fn is_authoring_transform_method(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    name == "author_markdown"
        || name.contains("markdown_transform")
        || name.contains("apply_markdown")
        || name.contains("run_authoring")
        || name.contains("to_operation")
}

pub(super) fn is_opaque_forwarding_name(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    (name.contains("opaque") || name.contains("receipt") || name.contains("event"))
        && (name.contains("forward")
            || name.contains("relay")
            || name.contains("consume")
            || name.contains("dispatch"))
}
