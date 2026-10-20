pub(super) fn is_paste_path(path: &syn::Path) -> bool {
    let segments = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    matches!(
        segments.as_slice(),
        [event, paste] if event == "Event" && paste == "Paste"
    ) || matches!(
        segments.as_slice(),
        [egui, event, paste] if egui == "egui" && event == "Event" && paste == "Paste"
    )
}
