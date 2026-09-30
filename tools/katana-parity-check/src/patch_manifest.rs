pub(crate) const PATCH_DOCUMENT: &str =
    include_str!("../../../docs/v0-1-0-katana-downstream-adapter-patch.md");

const TOML_FENCE_START: &str = "```toml";
const FENCE_END: &str = "```";
const EXPECTED_TOML_BLOCKS: usize = 1;
const DEV_DEPENDENCIES: &str = "dev-dependencies";
const KLE_CRATE: &str = "katana-language-editor";
const KLE_EGUI_CRATE: &str = "katana-language-editor-egui";
const KLE_PATH: &str = "../../../katana-language-editor/crates/katana-language-editor";
const KLE_EGUI_PATH: &str = "../../../katana-language-editor/crates/katana-language-editor-egui";

pub(crate) struct KatanaAdapterPatchManifestAudit;

impl KatanaAdapterPatchManifestAudit {
    pub(crate) fn validate() -> Result<(), String> {
        let blocks = TomlCodeBlocks::from_document(PATCH_DOCUMENT);
        if blocks.items.len() != EXPECTED_TOML_BLOCKS {
            return Err(format!(
                "expected {EXPECTED_TOML_BLOCKS} TOML code block in KatanA adapter patch draft, got {}",
                blocks.items.len()
            ));
        }
        let manifest = blocks
            .items
            .first()
            .ok_or_else(|| "KatanA adapter patch draft has no TOML block".to_string())?;
        let value: toml::Table = toml::from_str(manifest.source).map_err(|error| {
            format!(
                "KatanA adapter patch TOML block starting near line {} is invalid: {error}",
                manifest.start_line
            )
        })?;
        let dev_dependencies = value
            .get(DEV_DEPENDENCIES)
            .and_then(toml::Value::as_table)
            .ok_or_else(|| {
                "KatanA adapter patch TOML block is missing [dev-dependencies]".to_string()
            })?;
        validate_path_dependency(dev_dependencies, KLE_CRATE, KLE_PATH)?;
        validate_path_dependency(dev_dependencies, KLE_EGUI_CRATE, KLE_EGUI_PATH)
    }
}

fn validate_path_dependency(
    dev_dependencies: &toml::map::Map<String, toml::Value>,
    crate_name: &str,
    expected_path: &str,
) -> Result<(), String> {
    let Some(value) = dev_dependencies.get(crate_name) else {
        return Err(format!(
            "KatanA adapter patch TOML block is missing {crate_name}"
        ));
    };
    let Some(table) = value.as_table() else {
        return Err(format!(
            "KatanA adapter patch TOML dependency {crate_name} must be an inline table"
        ));
    };
    let Some(actual_path) = table.get("path").and_then(toml::Value::as_str) else {
        return Err(format!(
            "KatanA adapter patch TOML dependency {crate_name} is missing path"
        ));
    };
    if actual_path != expected_path {
        return Err(format!(
            "KatanA adapter patch TOML dependency {crate_name} path mismatch: expected {expected_path}, got {actual_path}"
        ));
    }
    Ok(())
}

struct TomlCodeBlocks<'a> {
    items: Vec<TomlCodeBlock<'a>>,
}

impl<'a> TomlCodeBlocks<'a> {
    fn from_document(document: &'a str) -> Self {
        let mut items = Vec::new();
        let mut line_offset = 0;
        let mut rest = document;
        while let Some(start) = rest.find(TOML_FENCE_START) {
            let before = &rest[..start];
            line_offset += before.lines().count();
            let after_start = &rest[start + TOML_FENCE_START.len()..];
            let after_newline = after_start.strip_prefix('\n').unwrap_or(after_start);
            line_offset += 1;
            let Some(end) = after_newline.find(FENCE_END) else {
                break;
            };
            let source = &after_newline[..end];
            items.push(TomlCodeBlock {
                source,
                start_line: line_offset + 1,
            });
            line_offset += source.lines().count() + 1;
            rest = &after_newline[end + FENCE_END.len()..];
        }
        Self { items }
    }
}

struct TomlCodeBlock<'a> {
    source: &'a str,
    start_line: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patch_draft_toml_block_records_local_kle_dependencies() -> Result<(), String> {
        KatanaAdapterPatchManifestAudit::validate()
    }

    #[test]
    fn extracts_all_toml_fenced_blocks() {
        let document = "```toml\n[dev-dependencies]\n```\ntext\n```toml\n[package]\n```\n";
        let blocks = TomlCodeBlocks::from_document(document);

        assert_eq!(blocks.items.len(), 2);
        assert!(blocks.items[0].source.contains("[dev-dependencies]"));
        assert!(blocks.items[1].source.contains("[package]"));
    }
}
