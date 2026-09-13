use std::collections::BTreeSet;

const SOURCE_PREFIX: &str = "crates/katana-ui/src/";
const KATANA_UI_CRATE_PREFIX: &str = "crates/katana-ui/";
const RELATIVE_PREFIXES: &[&str] = &["app/", "shell_ui/", "state/", "theme_bridge/", "views/"];
const REQUIREMENT_SOURCE_PREFIXES: &[(&str, &str)] = &[
    ("app_frame/", "crates/katana-ui/src/views/app_frame/"),
    ("top_bar/", "crates/katana-ui/src/views/top_bar/"),
    ("preview_pane/", "crates/katana-ui/src/preview_pane/"),
    ("tests/", "crates/katana-ui/tests/"),
];
const EDITOR_REQUIREMENTS_SECTION: &str = "## Editor Parity Requirements";

pub(crate) struct SourceRequirementsLedger;

impl SourceRequirementsLedger {
    pub(crate) fn validate_checked_in_coverage() -> Result<(), String> {
        Self::validate_coverage(
            include_bytes!("../../../../docs/v0-1-0-editor-requirements.md"),
            include_bytes!("../../../../docs/v0-1-0-katana-editor-source-universe.md"),
        )
    }

    pub(crate) fn validate_coverage(
        requirements: &[u8],
        source_universe: &[u8],
    ) -> Result<(), String> {
        let requirements = std::str::from_utf8(requirements)
            .map_err(|error| format!("editor-requirements input is not UTF-8: {error}"))?;
        let source_universe = std::str::from_utf8(source_universe)
            .map_err(|error| format!("source-universe input is not UTF-8: {error}"))?;
        let (_, requirements) = requirements
            .split_once(EDITOR_REQUIREMENTS_SECTION)
            .ok_or("editor-requirements omits the Editor Parity Requirements section")?;
        let documented_sources = Self::documented_rust_paths(source_universe);
        let documented_directories = Self::documented_directory_prefixes(source_universe);

        for source in Self::editor_requirement_source_paths(requirements) {
            let directly_documented = documented_sources.contains(&source);
            let covered_by_directory = documented_directories
                .iter()
                .any(|directory| source.starts_with(&format!("{directory}/")));
            if !directly_documented && !covered_by_directory {
                return Err(format!(
                    "source-universe omits editor requirement source: {source}"
                ));
            }
        }
        Ok(())
    }

    pub(super) fn normalized_source_paths(source_cell: &str) -> BTreeSet<String> {
        Self::documented_rust_paths(source_cell)
            .into_iter()
            .filter_map(|source| Self::normalize_editor_requirement_source(&source))
            .collect()
    }

    pub(super) fn source_tokens(source_cell: &str) -> BTreeSet<String> {
        Self::documented_rust_paths(source_cell)
    }

    pub(super) fn normalized_directory_prefixes(source_cell: &str) -> BTreeSet<String> {
        Self::documented_directory_prefixes(source_cell)
    }

    #[cfg(test)]
    pub(super) fn editor_requirement_source_paths_for_test(document: &str) -> BTreeSet<String> {
        Self::editor_requirement_source_paths(document)
    }

    fn editor_requirement_source_paths(document: &str) -> BTreeSet<String> {
        Self::documented_rust_paths(document)
            .into_iter()
            .filter_map(|source| Self::normalize_editor_requirement_source(&source))
            .collect()
    }

    fn normalize_editor_requirement_source(source: &str) -> Option<String> {
        if source.starts_with(SOURCE_PREFIX) {
            return Some(source.to_string());
        }
        for &(abbreviation, resolved_prefix) in REQUIREMENT_SOURCE_PREFIXES {
            if let Some(suffix) = source.strip_prefix(abbreviation) {
                return Some(format!("{resolved_prefix}{suffix}"));
            }
        }
        None
    }

    fn documented_rust_paths(document: &str) -> BTreeSet<String> {
        let mut paths = BTreeSet::new();
        let mut token = String::new();
        for character in document.chars().chain(std::iter::once(' ')) {
            if character.is_ascii_alphanumeric()
                || matches!(character, '_' | '-' | '.' | '/' | '{' | '}' | ',')
            {
                token.push(character);
                continue;
            }
            Self::record_token(&token, &mut paths);
            token.clear();
        }
        paths
    }

    fn documented_directory_prefixes(document: &str) -> BTreeSet<String> {
        let mut directories = BTreeSet::new();
        let mut token = String::new();
        for character in document.chars().chain(std::iter::once(' ')) {
            if character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | '/' | '*')
            {
                token.push(character);
                continue;
            }
            Self::record_directory_token(&token, &mut directories);
            token.clear();
        }
        directories
    }

    fn record_directory_token(token: &str, directories: &mut BTreeSet<String>) {
        let Some(directory) = token.strip_suffix("/**") else {
            return;
        };
        let directory = if directory.starts_with(KATANA_UI_CRATE_PREFIX) {
            directory.to_string()
        } else if RELATIVE_PREFIXES
            .iter()
            .any(|prefix| directory.starts_with(prefix))
        {
            format!("{SOURCE_PREFIX}{directory}")
        } else {
            return;
        };
        directories.insert(directory);
    }

    fn record_token(token: &str, paths: &mut BTreeSet<String>) {
        for path in Self::expand_braced_rust_paths(token) {
            Self::record_path(&path, paths);
        }
    }

    fn expand_braced_rust_paths(token: &str) -> Vec<String> {
        let Some(path) = token.strip_suffix(".rs") else {
            return Vec::new();
        };
        let Some(open) = path.find('{') else {
            return vec![token.to_string()];
        };
        let Some(close) = path[open + 1..].find('}') else {
            return Vec::new();
        };
        let close = open + close + 1;
        let prefix = &path[..open];
        let suffix = &path[close + 1..];
        path[open + 1..close]
            .split(',')
            .filter(|entry| !entry.is_empty())
            .map(|entry| format!("{prefix}{entry}{suffix}.rs"))
            .collect()
    }

    fn record_path(token: &str, paths: &mut BTreeSet<String>) {
        let Some(path) = token.strip_suffix(".rs") else {
            return;
        };
        let candidate = if let Some(path) = path.strip_prefix(SOURCE_PREFIX) {
            format!("{SOURCE_PREFIX}{path}.rs")
        } else if RELATIVE_PREFIXES
            .iter()
            .any(|prefix| path.starts_with(prefix))
        {
            format!("{SOURCE_PREFIX}{path}.rs")
        } else {
            format!("{path}.rs")
        };
        paths.insert(candidate);
    }
}
