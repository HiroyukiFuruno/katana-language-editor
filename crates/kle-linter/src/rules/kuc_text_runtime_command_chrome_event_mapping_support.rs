use std::path::Path;

pub(super) fn is_kuc_scroll_delegate(value: &syn::Expr) -> bool {
    let syn::Expr::Path(path) = value else {
        return false;
    };
    let segments = &path.path.segments;
    segments
        .first()
        .is_some_and(|segment| segment.ident == "EditorScrollControl")
        && segments
            .last()
            .is_some_and(|segment| segment.ident == "scroll_into_view")
}

pub(super) fn is_target(path: &Path) -> bool {
    path.file_name().is_some_and(|name| {
        matches!(
            name.to_str(),
            Some(
                "widget.rs"
                    | "kuc_command_chrome_composition.rs"
                    | "kuc_command_chrome_composition_events.rs"
                    | "kuc_command_chrome_composition_toolbar_events.rs"
                    | "kuc_command_chrome_composition_search_events.rs"
                    | "search_control.rs"
            )
        )
    })
}
