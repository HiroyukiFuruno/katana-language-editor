const FORBIDDEN_SYMBOLS: &[&str] = &[
    "SourceAddressProjectionLease",
    "SourceAddressSubmissionPort",
    "SourceAddressSubmission",
    "SourceAddressStrip",
    "EguiSourceAddressStripAdapter",
    "SourceAddressAction",
    "SourceAddressEvent",
    "SourceAddressPresentation",
    "SourceAddressEntry",
    "UrlTabState",
    "OpenUrl",
];

pub(super) fn is_forbidden_symbol(name: &str) -> bool {
    FORBIDDEN_SYMBOLS.contains(&name)
}

pub(super) fn is_source_address_context(name: &str) -> bool {
    let normalized = name.to_ascii_lowercase();
    normalized.contains("source_address") || normalized.contains("sourceaddress")
}

pub(super) fn is_url_conversion_helper(name: &str) -> bool {
    let normalized = name.to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "parse_file_url"
            | "file_url_to_path"
            | "decode_file_url_path"
            | "path_to_file_url"
            | "file_path_to_url"
            | "file_uri_to_path"
            | "path_to_file_uri"
    ) || (normalized.contains("file_url")
        && (normalized.contains("parse")
            || normalized.contains("convert")
            || normalized.contains("path")
            || normalized.contains("uri")))
}

pub(super) fn is_url_parse_or_conversion_path(names: &[String]) -> bool {
    names.iter().any(|name| name == "Url" || name == "Uri")
        || names.iter().any(|name| is_url_conversion_helper(name))
}

pub(super) fn is_direct_source_egui_path(names: &[String]) -> bool {
    names.first().is_some_and(|name| name == "egui")
        && names
            .last()
            .is_some_and(|name| matches!(name.as_str(), "TextEdit" | "Button" | "ImageButton"))
}
