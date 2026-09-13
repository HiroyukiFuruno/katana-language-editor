pub(super) const SEARCH: &str =
    "search find replace match matches regex navigate navigation cursor selection";
const MUTATION: &str = "replace replace_range replace_char_range replace_all push push_str insert remove clear truncate set_text set_content set_document";

pub(super) fn has_word(name: &str, candidates: &str) -> bool {
    let name = name.to_ascii_lowercase();
    candidates
        .split_whitespace()
        .any(|candidate| name.contains(candidate))
}

pub(super) fn is_constant_identifier(name: &str) -> bool {
    name.chars().all(|character| {
        character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_'
    })
}

pub(super) fn is_local_semantic_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    if is_low_level_editing_name(name) {
        return false;
    }
    let subject = has_word(name, "document content text")
        || (has_word(name, "string") && !lower.contains("strings"));
    has_word(name, SEARCH)
        && (subject
            || has_word(
                name,
                "state model result results range ranges index indices query algorithm",
            ))
}

pub(super) fn is_forbidden_operation_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "search"
            | "find"
            | "replace"
            | "replace_all"
            | "find_next"
            | "find_previous"
            | "find_matches"
            | "compute_matches"
            | "regex_pattern_is_valid"
    )
}

pub(super) fn is_low_level_editing_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    matches!(lower.as_str(), "replace_range" | "replace_char_range")
        || lower.contains("grapheme")
        || lower.contains("character_range")
}

pub(super) fn mutation(name: &str) -> bool {
    MUTATION
        .split_whitespace()
        .any(|candidate| candidate == name)
}
