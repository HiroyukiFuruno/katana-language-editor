use std::collections::BTreeSet;

use super::source_requirement_alias_ledger::RequirementSourceAliasLedger;

const PARITY_SECTION: &str = "## Editor Parity Requirements";
const SECTION_END: &str = "### Non-Aggregate Authoring Leaf Catalog";
const RUST_SUFFIX: &str = ".rs";
const SOURCE_CELL_INDEX: usize = 3;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RequirementSourceRow {
    pub(super) requirement_id: String,
    pub(super) source_cell: String,
}

impl RequirementSourceAliasLedger {
    pub(super) fn requirement_source_rows(
        requirements: &str,
    ) -> Result<Vec<RequirementSourceRow>, String> {
        let (_, section) = requirements
            .split_once(PARITY_SECTION)
            .ok_or("editor requirements omit the parity section")?;
        let table = section
            .split_once(SECTION_END)
            .map_or(section, |(table, _)| table);
        table
            .lines()
            .filter(|line| line.starts_with("| `"))
            .map(|line| {
                let fields = line.split('|').collect::<Vec<_>>();
                let requirement_id = fields
                    .get(1)
                    .ok_or("requirement row omits requirement ID field")?
                    .trim()
                    .trim_matches('`');
                if requirement_id.is_empty() {
                    return Err("requirement row has an empty requirement ID".to_string());
                }
                let source_cell = fields
                    .get(SOURCE_CELL_INDEX)
                    .ok_or("requirement row omits source cell field")?
                    .trim();
                Ok(RequirementSourceRow {
                    requirement_id: requirement_id.to_string(),
                    source_cell: source_cell.to_string(),
                })
            })
            .collect()
    }

    pub(super) fn short_references(
        requirements: &str,
    ) -> Result<BTreeSet<(String, String)>, String> {
        let mut references = BTreeSet::new();
        for row in Self::requirement_source_rows(requirements)? {
            for reference in Self::short_references_in(&row.source_cell) {
                references.insert((row.requirement_id.clone(), reference));
            }
        }
        Ok(references)
    }

    fn short_references_in(source_cell: &str) -> Vec<String> {
        let bytes = source_cell.as_bytes();
        let mut references = Vec::new();
        let mut start = 0;
        while start < bytes.len() {
            if !bytes[start].is_ascii_alphabetic()
                || start > 0 && !Self::is_boundary(bytes[start - 1])
            {
                start += 1;
                continue;
            }
            let mut end = start + 1;
            while end < bytes.len()
                && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_' || bytes[end] == b'-')
            {
                end += 1;
            }
            if source_cell[end..].starts_with(RUST_SUFFIX) {
                let suffix = end + RUST_SUFFIX.len();
                if suffix == bytes.len() || Self::is_boundary(bytes[suffix]) {
                    references.push(source_cell[start..suffix].to_string());
                }
            }
            start = end;
        }
        references
    }

    fn is_boundary(byte: u8) -> bool {
        !byte.is_ascii_alphanumeric() && !matches!(byte, b'_' | b'-' | b'.' | b'/' | b'{' | b'}')
    }
}
