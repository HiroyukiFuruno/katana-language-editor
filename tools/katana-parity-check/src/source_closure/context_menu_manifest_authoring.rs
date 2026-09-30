use super::super::super::DIRECT_AUTHORING_COUNT;

pub(super) fn derive_direct_authoring_ops(source: &str) -> Result<Vec<String>, String> {
    let mut variants = Vec::new();
    let mut search_start = 0;
    while let Some(relative_start) = source[search_start..].find("Self::author_button(") {
        let start = search_start + relative_start;
        let body_start = start + "Self::author_button(".len();
        let body_end = source[body_start..]
            .find(");")
            .map(|offset| body_start + offset)
            .ok_or_else(|| "author_button call is not terminated".to_string())?;
        let body = &source[body_start..body_end];
        let marker = "MarkdownAuthoringOp::";
        let marker_start = body.find(marker).ok_or_else(|| {
            "author_button call is missing MarkdownAuthoringOp variant".to_string()
        })?;
        let variant = body[marker_start + marker.len()..]
            .split(|ch: char| !ch.is_ascii_alphanumeric())
            .next()
            .unwrap_or_default();
        if variant.is_empty() || variant == "CodeBlock" {
            return Err("direct authoring source contains an invalid or container variant".into());
        }
        if variants.iter().any(|known| known == variant) {
            return Err(format!(
                "fixed direct authoring source contains duplicate variant {variant}"
            ));
        }
        variants.push(variant.to_string());
        search_start = body_end + 2;
    }
    if variants.len() != DIRECT_AUTHORING_COUNT {
        return Err(format!(
            "fixed direct authoring inventory must contain {DIRECT_AUTHORING_COUNT} leaves, got {}",
            variants.len()
        ));
    }
    Ok(variants)
}
