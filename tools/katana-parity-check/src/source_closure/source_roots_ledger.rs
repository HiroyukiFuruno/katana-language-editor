use std::collections::BTreeSet;
use std::path::Path;

const SOURCE_PREFIX: &str = "crates/katana-ui/src/";
#[cfg(test)]
#[path = "source_roots_vendor_tests.rs"]
mod vendor_tests;
const RELATIVE_PREFIXES: &[&str] = &["app/", "shell_ui/", "state/", "theme_bridge/", "views/"];
const REQUIRED_DIRECTORY_MARKERS: &[(&str, &str)] = &[
    (
        "crates/katana-ui/src/views/panels/editor/**",
        "crates/katana-ui/src/views/panels/editor",
    ),
    (
        "crates/katana-ui/tests/integration/editor/**",
        "crates/katana-ui/tests/integration/editor",
    ),
    (
        "crates/katana-ui/src/views/top_bar/tab_bar/**",
        "crates/katana-ui/src/views/top_bar/tab_bar",
    ),
    (
        "crates/katana-ui/src/views/modals/search_tabs/**",
        "crates/katana-ui/src/views/modals/search_tabs",
    ),
];

pub(super) fn validate_documented_directories(
    source_universe: &[u8],
    directory_roots: &[String],
) -> Result<(), String> {
    let document = std::str::from_utf8(source_universe)
        .map_err(|error| format!("source-universe input is not UTF-8: {error}"))?;
    for &(marker, required_root) in REQUIRED_DIRECTORY_MARKERS {
        if document.contains(marker) && !directory_roots.iter().any(|root| root == required_root) {
            return Err(format!(
                "source-root manifest omits documented directory root: {required_root}"
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_documented_file_roots(
    source_universe: &[u8],
    file_roots: &[String],
) -> Result<(), String> {
    let document = std::str::from_utf8(source_universe)
        .map_err(|error| format!("source-universe input is not UTF-8: {error}"))?;
    let documented = documented_rust_paths(document);
    for root in file_roots {
        if !documented.contains(root) {
            return Err(format!(
                "source-root manifest contains undocumented file root: {root}"
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_documented_paths(
    source_universe: &[u8],
    katana_root: &Path,
    resolved_paths: &BTreeSet<String>,
) -> Result<(), String> {
    let document = std::str::from_utf8(source_universe)
        .map_err(|error| format!("source-universe input is not UTF-8: {error}"))?;
    for reference in documented_rust_paths(document) {
        if katana_root.join(&reference).is_file() && !resolved_paths.contains(&reference) {
            return Err(format!(
                "source-root manifest omits documented source path: {reference}"
            ));
        }
    }
    Ok(())
}

pub(super) fn documented_rust_paths(document: &str) -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    let mut token = String::new();
    for character in document.chars().chain(std::iter::once(' ')) {
        if character.is_ascii_alphanumeric()
            || matches!(character, '_' | '-' | '.' | '/' | '{' | '}' | ',')
        {
            token.push(character);
            continue;
        }
        record_token(&token, &mut paths);
        token.clear();
    }
    paths
}

fn record_token(token: &str, paths: &mut BTreeSet<String>) {
    for path in expand_braced_rust_paths(token) {
        record_path(&path, paths);
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{REQUIRED_DIRECTORY_MARKERS, validate_documented_directories};

    #[test]
    fn documented_directory_markers_require_exact_directory_roots() {
        let roots = vec!["crates/katana-ui/src/views/panels".to_string()];

        for &(marker, required_root) in REQUIRED_DIRECTORY_MARKERS {
            assert!(matches!(
                validate_documented_directories(marker.as_bytes(), &roots),
                Err(error) if error.contains(required_root)
            ));
        }
    }

    #[test]
    fn documented_rust_paths_expand_braced_source_routes() {
        let paths = super::documented_rust_paths(
            "crates/katana-ui/src/app/action/{dispatch,dispatch_secondary}.rs",
        );

        assert!(paths.contains("crates/katana-ui/src/app/action/dispatch.rs"));
        assert!(paths.contains("crates/katana-ui/src/app/action/dispatch_secondary.rs"));
    }

    #[test]
    fn braced_documented_source_omission_rejects_manifest_expansion()
    -> Result<(), Box<dyn std::error::Error>> {
        let suffix = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let root = std::env::temp_dir().join(format!(
            "katana-parity-braced-source-roots-{}-{suffix}",
            std::process::id()
        ));
        let action_root = root.join("crates/katana-ui/src/app/action");
        fs::create_dir_all(&action_root)?;
        fs::write(action_root.join("dispatch.rs"), "fn dispatch() {}")?;
        fs::write(
            action_root.join("dispatch_secondary.rs"),
            "fn dispatch_secondary() {}",
        )?;
        let resolved = BTreeSet::from(["crates/katana-ui/src/app/action/dispatch.rs".to_string()]);

        let rejected = matches!(
            super::validate_documented_paths(
                b"crates/katana-ui/src/app/action/{dispatch,dispatch_secondary}.rs",
                &root,
                &resolved,
            ),
            Err(error) if error.contains("dispatch_secondary.rs")
        );

        fs::remove_dir_all(&root)?;
        assert!(rejected);
        Ok(())
    }
}
