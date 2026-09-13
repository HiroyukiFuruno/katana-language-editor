struct Expected {
    surface_span: String,
    menu_span: String,
    menu_path: String,
    leaves: Vec<ExpectedLeaf>,
}

struct ExpectedLeaf {
    path: String,
    span: String,
    condition: String,
    labels: Vec<String>,
}

impl Expected {
    fn from_source(root: &Path) -> Result<Self, ContextMenuManifestError> {
        let context = read_source(root, CONTEXT_SOURCE)?;
        let code = read_source(root, CODE_SOURCE)?;
        let ingest = read_source(root, INGEST_SOURCE)?;
        let mut source = context;
        source.push('\n');
        source.push_str(&ingest);
        let menu_path = digest(MENU_PATH);
        let surface_span = span(CONTEXT_SOURCE, &source, "response.context_menu")?;
        let mut leaves = Vec::with_capacity(LEAF_COUNT);
        push_leaf(
            &mut leaves,
            &source,
            "save",
            "AppAction::SaveDocument",
            "always",
            &root.join(LOCALES),
            "action.save",
        )?;
        push_leaf(
            &mut leaves,
            &source,
            "format",
            "AppAction::FormatMarkdownFile",
            "editable && extension_is_md_or_markdown",
            &root.join(LOCALES),
            "action.format_markdown_file",
        )?;
        for variant in direct_authoring_variants(&source)? {
            let name = snake_case(variant);
            push_leaf(
                &mut leaves,
                &source,
                &name,
                &format!("MarkdownAuthoringOp::{variant}"),
                "selection_or_structural",
                &root.join(LOCALES),
                &format!("search.command_author_{name}"),
            )?;
        }
        for kind in code_block_kinds(&code)? {
            let path = format!("{MENU_PATH}/code_block/{}", kind.to_ascii_lowercase());
            leaves.push(ExpectedLeaf {
                path: digest(&path),
                span: span(CODE_SOURCE, &code, &format!("Self::{kind} =>"))?,
                condition: digest("always"),
                labels: vec![code_label_digest(&code, &kind)?],
            });
        }
        push_leaf(
            &mut leaves,
            &source,
            "image_file",
            "command_ingest_image_file",
            "always",
            &root.join(LOCALES),
            "search.command_ingest_image_file",
        )?;
        push_leaf(
            &mut leaves,
            &source,
            "clipboard_image",
            "command_ingest_clipboard_image",
            "clipboard_image_available",
            &root.join(LOCALES),
            "search.command_ingest_clipboard_image",
        )?;
        Ok(Self {
            surface_span: surface_span.clone(),
            menu_span: surface_span,
            menu_path,
            leaves,
        })
    }

    fn validate(&self, raw: &RawManifest) -> Result<(), ContextMenuManifestError> {
        if raw.leaves.len() != LEAF_COUNT
            || raw.surface.source_span_digest != self.surface_span
            || raw.menu.source_span_digest != self.menu_span
            || raw.menu.path_digest != self.menu_path
        {
            return Err(ContextMenuManifestError::Invalid);
        }
        let mut paths = BTreeSet::new();
        let mut spans = BTreeSet::new();
        for (actual, expected) in raw.leaves.iter().zip(&self.leaves) {
            if actual.role != "Button"
                || actual.path_digest != expected.path
                || actual.source_span_digest != expected.span
                || actual.enabled_condition_digest != expected.condition
                || actual.parent_path_digest != self.menu_path
                || actual.locale_label_digests != expected.labels
                || !paths.insert(actual.path_digest.clone())
                || !spans.insert(actual.source_span_digest.clone())
                || actual.locale_label_digests.is_empty()
                || actual
                    .locale_label_digests
                    .windows(2)
                    .any(|pair| pair[0] >= pair[1])
                || actual
                    .locale_label_digests
                    .iter()
                    .any(|value| !is_digest(value))
            {
                return Err(ContextMenuManifestError::Invalid);
            }
        }
        Ok(())
    }
}

fn push_leaf(
    leaves: &mut Vec<ExpectedLeaf>,
    source: &str,
    name: &str,
    marker: &str,
    condition: &str,
    locales: &Path,
    key: &str,
) -> Result<(), ContextMenuManifestError> {
    leaves.push(ExpectedLeaf {
        path: digest(&format!("{MENU_PATH}/{name}")),
        span: span(CONTEXT_SOURCE, source, marker)?,
        condition: digest(condition),
        labels: locale_digests(locales, key)?,
    });
    Ok(())
}
