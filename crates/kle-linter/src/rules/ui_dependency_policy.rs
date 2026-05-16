pub(super) struct UiDependencyPolicy;

impl UiDependencyPolicy {
    pub(super) fn is_ui_dependency(name: &str) -> bool {
        let lower = name.to_ascii_lowercase();
        lower.ends_with("-ui")
            || lower.ends_with("_ui")
            || matches!(
                lower.as_str(),
                "dioxus" | "eframe" | "egui" | "floem" | "iced" | "leptos" | "tauri" | "yew"
            )
    }
}
