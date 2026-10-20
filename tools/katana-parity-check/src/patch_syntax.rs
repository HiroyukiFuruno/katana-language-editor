pub(crate) const PATCH_DOCUMENT: &str =
    include_str!("../../../docs/v0-1-0-katana-downstream-adapter-patch.md");

const RUST_FENCE_START: &str = "```rust";
const FENCE_END: &str = "```";
const EXPECTED_RUST_BLOCKS: usize = 2;

pub(crate) struct KatanaAdapterPatchSyntaxAudit;

impl KatanaAdapterPatchSyntaxAudit {
    pub(crate) fn validate() -> Result<(), String> {
        let blocks = RustCodeBlocks::from_document(PATCH_DOCUMENT);
        if blocks.items.len() != EXPECTED_RUST_BLOCKS {
            return Err(format!(
                "expected {EXPECTED_RUST_BLOCKS} Rust code blocks in KatanA adapter patch draft, got {}",
                blocks.items.len()
            ));
        }
        for block in blocks.items {
            syn::parse_file(block.source).map_err(|error| {
                format!(
                    "KatanA adapter patch Rust block starting near line {} is invalid: {error}",
                    block.start_line
                )
            })?;
        }
        Ok(())
    }
}

struct RustCodeBlocks<'a> {
    items: Vec<RustCodeBlock<'a>>,
}

impl<'a> RustCodeBlocks<'a> {
    fn from_document(document: &'a str) -> Self {
        let mut items = Vec::new();
        let mut line_offset = 0;
        let mut rest = document;
        while let Some(start) = rest.find(RUST_FENCE_START) {
            let before = &rest[..start];
            line_offset += before.lines().count();
            let after_start = &rest[start + RUST_FENCE_START.len()..];
            let after_newline = after_start.strip_prefix('\n').unwrap_or(after_start);
            line_offset += 1;
            let Some(end) = after_newline.find(FENCE_END) else {
                break;
            };
            let source = &after_newline[..end];
            items.push(RustCodeBlock {
                source,
                start_line: line_offset + 1,
            });
            line_offset += source.lines().count() + 1;
            rest = &after_newline[end + FENCE_END.len()..];
        }
        Self { items }
    }
}

struct RustCodeBlock<'a> {
    source: &'a str,
    start_line: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patch_draft_rust_blocks_parse_as_rust_files() -> Result<(), String> {
        KatanaAdapterPatchSyntaxAudit::validate()
    }

    #[test]
    fn extracts_all_rust_fenced_blocks() {
        let document = "```rust\nfn a() {}\n```\ntext\n```rust\nfn b() {}\n```\n";
        let blocks = RustCodeBlocks::from_document(document);

        assert_eq!(blocks.items.len(), 2);
        assert!(blocks.items[0].source.contains("fn a"));
        assert!(blocks.items[1].source.contains("fn b"));
    }
}
