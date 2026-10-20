use std::collections::BTreeSet;
use std::path::Path;

use serde_json::Value;

use super::super::super::super::fingerprint::sha256_hex;
use super::super::super::{
    CONTEXT_MENU_SOURCE, ContextMenuLeaf, LOCALES_DIRECTORY, LOCALES_METADATA_FILE, ROLE_LEAF,
};
use super::super::LocaleSourceInput;

pub(super) fn code_block_label_digest(source: &str, kind: &str) -> Result<String, String> {
    let marker = format!("Self::{kind} =>");
    let start = source
        .find(&marker)
        .ok_or_else(|| format!("missing CodeBlockKind label for {kind}"))?
        + marker.len();
    let label = source[start..]
        .split('"')
        .nth(1)
        .ok_or_else(|| format!("invalid CodeBlockKind label for {kind}"))?;
    Ok(sha256_hex(label.as_bytes()))
}

pub(super) fn snake_case(value: &str) -> String {
    value
        .chars()
        .enumerate()
        .fold(String::new(), |mut out, (index, ch)| {
            if ch.is_ascii_uppercase() && index != 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
            out
        })
}

pub(super) fn add_digest_leaf(
    leaves: &mut Vec<ContextMenuLeaf>,
    source: &str,
    path: &str,
    source_marker: &str,
    condition: &str,
    locale_label_digests: Vec<String>,
) -> Result<(), String> {
    leaves.push(ContextMenuLeaf {
        path_digest: path_digest(path),
        source_span_digest: span_digest(CONTEXT_MENU_SOURCE, source, source_marker),
        role: ROLE_LEAF.into(),
        parent_path_digest: path_digest("editor.text_surface/context_menu"),
        enabled_condition_digest: sha256_hex(condition.as_bytes()),
        locale_label_digests,
    });
    Ok(())
}

pub(super) fn locale_digests(
    locales: &[(String, Vec<u8>)],
    key: &str,
) -> Result<Vec<String>, String> {
    let mut digests = BTreeSet::new();
    for (locale, bytes) in locales {
        let value: Value = serde_json::from_slice(bytes)
            .map_err(|error| format!("fixed locale {locale} is invalid JSON: {error}"))?;
        let mut current = &value;
        for part in key.split('.') {
            current = current
                .get(part)
                .ok_or_else(|| format!("fixed locale {locale} is missing {key}"))?;
        }
        let label = current
            .as_str()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("fixed locale {locale} has invalid {key}"))?;
        digests.insert(sha256_hex(label.as_bytes()));
    }
    Ok(digests.into_iter().collect())
}

pub(super) fn span_digest(path: &str, source: &str, marker: &str) -> String {
    let line = source
        .lines()
        .position(|line| line.contains(marker))
        .map_or(0, |line| line + 1);
    sha256_hex(format!("katana:{path}:{line}-{line}").as_bytes())
}

pub(super) fn path_digest(path: &str) -> String {
    sha256_hex(path.as_bytes())
}

pub(super) fn read_utf8(root: &Path, relative: &str) -> Result<String, String> {
    let bytes = std::fs::read(root.join(relative))
        .map_err(|error| format!("cannot read fixed KatanA source {relative}: {error}"))?;
    String::from_utf8(bytes)
        .map_err(|error| format!("fixed source {relative} is not UTF-8: {error}"))
}

pub(super) fn read_locales(root: &Path) -> Result<Vec<LocaleSourceInput>, String> {
    let mut paths = std::fs::read_dir(root.join(LOCALES_DIRECTORY))
        .map_err(|error| format!("cannot read fixed locale directory: {error}"))?
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|error| error.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    paths.retain(|path| {
        path.extension()
            .is_some_and(|extension| extension == "json")
            && path
                .file_name()
                .is_none_or(|name| name != LOCALES_METADATA_FILE)
    });
    paths.sort_unstable();
    paths
        .into_iter()
        .map(|path| {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| "fixed locale filename is not UTF-8".to_string())?
                .to_owned();
            let bytes = std::fs::read(&path)
                .map_err(|error| format!("cannot read fixed locale {name}: {error}"))?;
            Ok((name, bytes))
        })
        .collect()
}
