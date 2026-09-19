pub(super) struct ColorLiteralPatterns;

impl ColorLiteralPatterns {
    pub(super) fn path_segments(path: &syn::Path) -> Vec<String> {
        path.segments
            .iter()
            .map(|it| it.ident.to_string())
            .collect()
    }

    pub(super) fn is_color_constant_path(names: &[String], last: &str) -> bool {
        Self::has_color_type(names) && Self::is_constant_name(last)
    }

    pub(super) fn is_color_constructor_path(names: &[String]) -> bool {
        let Some(last) = names.last() else {
            return false;
        };
        Self::has_color_type(names)
            && matches!(
                last.as_str(),
                "from_rgb"
                    | "from_gray"
                    | "from_rgba_unmultiplied"
                    | "from_rgba_premultiplied"
                    | "rgb"
                    | "rgb8"
                    | "rgba"
                    | "rgba8"
            )
    }

    pub(super) fn is_literal_arg(arg: &syn::Expr) -> bool {
        matches!(arg, syn::Expr::Lit(_))
    }

    pub(super) fn is_color_string(value: &str) -> bool {
        let trimmed = value.trim();
        Self::is_hex_color(trimmed) || Self::is_css_color_function(trimmed)
    }

    fn has_color_type(names: &[String]) -> bool {
        names.iter().any(|it| it == "Color32" || it == "Color")
    }

    fn is_constant_name(name: &str) -> bool {
        name.chars().any(|it| it.is_ascii_uppercase())
            && !name.chars().any(|it| it.is_ascii_lowercase())
    }

    fn is_hex_color(value: &str) -> bool {
        let Some(hex) = value.strip_prefix('#') else {
            return false;
        };
        matches!(hex.len(), 3 | 4 | 6 | 8) && hex.chars().all(|it| it.is_ascii_hexdigit())
    }

    fn is_css_color_function(value: &str) -> bool {
        let lower = value.to_ascii_lowercase();
        ["rgb(", "rgba(", "hsl(", "hsla("]
            .into_iter()
            .any(|prefix| lower.starts_with(prefix))
    }
}

#[cfg(test)]
mod tests {
    use super::ColorLiteralPatterns;

    #[test]
    fn recognizes_all_required_css_color_function_prefixes() {
        for value in [
            "rgb(1, 2, 3)",
            "rgba(1, 2, 3, 0.5)",
            "hsl(120, 50%, 50%)",
            "hsla(120, 50%, 50%, 0.5)",
            "  HSL(120 50% 50%)\r\n",
            "\tHsLa(120 50% 50% / 0.5) ",
        ] {
            assert!(ColorLiteralPatterns::is_color_string(value), "{value:?}");
        }
    }

    #[test]
    fn unrelated_strings_are_not_color_function_literals() {
        for value in ["hsl", "hsla", "my_hsl(120)", "text hsl(120)", "#xyz"] {
            assert!(!ColorLiteralPatterns::is_color_string(value), "{value:?}");
        }
    }
}
