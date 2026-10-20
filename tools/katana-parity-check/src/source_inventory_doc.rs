use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

const INVENTORY_SECTION: &str = "## Source Inventory";
const INVENTORY_FEATURE_ID_SECTION: &str = "## Source Inventory Feature IDs";
const MIN_SOURCE_INVENTORY_COLUMNS: usize = 4;
const SOURCE_FILE_COLUMN: usize = 0;
const REQUIREMENT_ROWS_COLUMN: usize = 1;
const INTEGRATION_FUNCTIONS_COLUMN: usize = 2;
const STATUS_COLUMN: usize = 3;

#[derive(Default, Debug)]
pub(crate) struct SourceInventoryRow {
    pub(crate) integration_functions: Vec<String>,
    pub(crate) feature_ids: Vec<String>,
}

pub(crate) struct SourceInventoryDocument;

impl SourceInventoryDocument {
    pub(crate) fn parse_source_inventory(
        document: &str,
        repo_root: &Path,
        matrix_feature_ids: &HashSet<String>,
    ) -> Result<HashMap<String, SourceInventoryRow>, String> {
        let mut in_section = false;
        let mut rows = HashMap::new();

        for line in document.lines() {
            if line.starts_with(INVENTORY_SECTION) {
                in_section = true;
                continue;
            }
            if !in_section {
                continue;
            }
            if line.starts_with("## ") {
                break;
            }

            let Some(row) = parse_inventory_row(line, repo_root, matrix_feature_ids)? else {
                continue;
            };
            rows.insert(row.file, row.inventory);
        }

        if rows.is_empty() {
            return Err("source inventory table is empty or missing".to_string());
        }

        Ok(rows)
    }

    pub(crate) fn parse_source_inventory_feature_ids(
        document: &str,
    ) -> Result<HashSet<String>, String> {
        let mut in_section = false;
        let mut ids = HashSet::new();

        for line in document.lines() {
            if line.starts_with(INVENTORY_FEATURE_ID_SECTION) {
                in_section = true;
                continue;
            }
            if !in_section {
                continue;
            }
            if line.starts_with("## ") {
                break;
            }

            for token in parse_backticked_tokens(line) {
                ids.insert(token);
            }
        }

        if ids.is_empty() {
            return Err("source inventory feature-id section is missing or empty".to_string());
        }

        Ok(ids)
    }
}

struct ParsedInventoryRow {
    file: String,
    inventory: SourceInventoryRow,
}

fn parse_inventory_row(
    line: &str,
    repo_root: &Path,
    matrix_feature_ids: &HashSet<String>,
) -> Result<Option<ParsedInventoryRow>, String> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') || trimmed.contains("---") || trimmed.contains("File") {
        return Ok(None);
    }

    let columns: Vec<&str> = trimmed
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .collect();

    if columns.len() < MIN_SOURCE_INVENTORY_COLUMNS {
        return Err(format!(
            "invalid source inventory row, expected >=4 columns: {trimmed}"
        ));
    }

    let file = normalize_path(columns[SOURCE_FILE_COLUMN], repo_root);
    if file.is_empty() {
        return Ok(None);
    }
    let integration_functions = parse_backticked_tokens(columns[INTEGRATION_FUNCTIONS_COLUMN]);
    let requirement_rows = parse_backticked_tokens(columns[REQUIREMENT_ROWS_COLUMN]);
    if requirement_rows.is_empty()
        && !columns[STATUS_COLUMN].contains("historical planned path is not evidence")
    {
        return Err(format!(
            "source inventory row has no mapped requirement rows: {file}"
        ));
    }

    if let Some(unknown_id) = requirement_rows
        .iter()
        .find(|row_id| !matrix_feature_ids.contains(*row_id))
    {
        return Err(format!(
            "mapped requirement rows contain undefined feature id '{unknown_id}' in {file}"
        ));
    }

    Ok(Some(ParsedInventoryRow {
        file,
        inventory: SourceInventoryRow {
            integration_functions,
            feature_ids: requirement_rows,
        },
    }))
}

fn normalize_path(raw: &str, repo_root: &Path) -> String {
    let trimmed = raw.trim().trim_matches('`');
    let path = Path::new(trimmed);

    if !path.is_absolute() {
        return normalized(path);
    }

    path.strip_prefix(repo_root)
        .map_or_else(|_| normalized(path), normalized)
}

fn normalized(path: impl AsRef<Path>) -> String {
    path.as_ref()
        .to_string_lossy()
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_string()
}

fn parse_backticked_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = text;
    while let Some(start) = current.find('`') {
        current = &current[start + 1..];
        let Some(end) = current.find('`') else {
            break;
        };

        let token = current[..end].trim();
        if !token.is_empty() {
            tokens.push(token.to_string());
        }
        current = &current[end + 1..];
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_backticked_tokens_works() {
        let values = parse_backticked_tokens("`test_a`, `test_b` and `test_c`");
        assert_eq!(values, vec!["test_a", "test_b", "test_c"]);
    }

    #[test]
    fn parse_source_inventory_requires_data() -> Result<(), String> {
        let doc = "## Source Inventory Feature IDs\n- `text-editing`\n\n## Source Inventory\n| File | Requirement rows | Integration function(s) | Notes |\n| --- | --- | --- | --- |\n| `a.rs` | `text-editing` | `test_a` | ok |\n";
        let ids = SourceInventoryDocument::parse_source_inventory_feature_ids(doc)?;
        let parsed = SourceInventoryDocument::parse_source_inventory(doc, Path::new("/"), &ids)?;
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed["a.rs"].integration_functions, ["test_a".to_string()]);
        assert_eq!(parsed["a.rs"].feature_ids, ["text-editing".to_string()]);
        Ok(())
    }

    #[test]
    fn parse_source_inventory_rejects_unknown_feature_mapping() -> Result<(), String> {
        let matrix_ids: HashSet<String> = ["text-editing"].into_iter().map(String::from).collect();
        let doc = "## Source Inventory Feature IDs\n- `text-editing`\n\n## Source Inventory\n| File | Requirement rows | Integration function(s) | Notes |\n| --- | --- | --- | --- |\n| `a.rs` | `unknown-id` | `test_a` | ok |\n";

        let error =
            match SourceInventoryDocument::parse_source_inventory(doc, Path::new("/"), &matrix_ids)
            {
                Ok(_) => return Err("unknown feature mapping should fail".to_string()),
                Err(error) => error,
            };
        assert!(
            error.contains(
                "mapped requirement rows contain undefined feature id 'unknown-id' in a.rs"
            )
        );
        Ok(())
    }
}
