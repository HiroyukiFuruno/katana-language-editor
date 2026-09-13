use std::collections::{BTreeMap, BTreeSet};

use super::artifact_model::SourceClosureArtifact;
use super::requirement_source_inventory::{
    RequirementSourceInventoryEntry, RequirementSourceReference,
};
use super::source_requirement_alias_ledger::AliasEntry;
use super::source_requirements_ledger::SourceRequirementsLedger;

#[derive(Default)]
pub(super) struct Facts {
    pub(super) provenance: BTreeSet<String>,
    pub(super) unresolved: BTreeSet<String>,
}

pub(super) struct RowSources {
    pub(super) references: Vec<RequirementSourceReference>,
    pub(super) unresolved: BTreeSet<String>,
    pub(super) entries: BTreeMap<(String, String), Facts>,
}

pub(super) fn row(
    id: &str,
    cell: &str,
    source: &SourceClosureArtifact,
    aliases: Option<&Vec<&AliasEntry>>,
) -> RowSources {
    let mut refs: BTreeMap<(String, Option<String>), RequirementSourceReference> = BTreeMap::new();
    let mut unresolved = BTreeSet::new();
    let mut entries = BTreeMap::new();
    for token in SourceRequirementsLedger::source_tokens(cell) {
        for path in SourceRequirementsLedger::normalized_source_paths(&token) {
            let count = source.files.iter().filter(|file| file.path == path).count();
            let status = match count {
                0 => {
                    unresolved.insert(format!("unscanned source: {path}"));
                    "unscanned"
                }
                1 => {
                    add(&mut entries, id, &path, &token);
                    "resolved"
                }
                _ => {
                    unresolved.insert(format!("duplicate scanned source: {path}"));
                    "duplicate_scanned"
                }
            };
            refs.insert(
                (token.clone(), Some(path.clone())),
                RequirementSourceReference {
                    raw_reference: token.clone(),
                    normalized_path: Some(path),
                    status: status.to_string(),
                },
            );
        }
    }
    for directory in SourceRequirementsLedger::normalized_directory_prefixes(cell) {
        let prefix = format!("{directory}/");
        let paths = source
            .files
            .iter()
            .filter(|file| file.path.starts_with(&prefix))
            .map(|file| file.path.clone())
            .collect::<BTreeSet<_>>();
        if paths.is_empty() {
            unresolved.insert(format!("unexpanded directory: {directory}/**"));
        }
        for path in paths {
            let count = source.files.iter().filter(|file| file.path == path).count();
            let status = if count == 1 {
                add(&mut entries, id, &path, &format!("{directory}/**"));
                "resolved"
            } else {
                unresolved.insert(format!("duplicate scanned source: {path}"));
                "duplicate_scanned"
            };
            refs.insert(
                (format!("{directory}/**"), Some(path.clone())),
                RequirementSourceReference {
                    raw_reference: format!("{directory}/**"),
                    normalized_path: Some(path),
                    status: status.to_string(),
                },
            );
        }
    }
    for alias in aliases.into_iter().flatten() {
        let count = source
            .files
            .iter()
            .filter(|file| file.path == alias.source_path)
            .count();
        if count != 1 {
            unresolved.insert(format!("alias source unavailable: {}", alias.source_path));
        }
        if count == 1 {
            add(&mut entries, id, &alias.source_path, &alias.short_reference);
        }
        refs.insert(
            (
                alias.short_reference.clone(),
                Some(alias.source_path.clone()),
            ),
            RequirementSourceReference {
                raw_reference: alias.short_reference.clone(),
                normalized_path: Some(alias.source_path.clone()),
                status: if count == 1 { "resolved" } else { "unresolved" }.to_string(),
            },
        );
    }
    for token in SourceRequirementsLedger::source_tokens(cell) {
        if SourceRequirementsLedger::normalized_source_paths(&token).is_empty()
            && !refs
                .keys()
                .any(|(raw_reference, _)| raw_reference == &token)
        {
            unresolved.insert(format!("unknown source reference: {token}"));
        }
    }
    if cell.contains("::") {
        unresolved.insert(format!("unclassified semantic reference: {cell}"));
    }
    if refs.is_empty() && unresolved.is_empty() {
        unresolved.insert(if cell.trim().is_empty() {
            "source reference is missing".into()
        } else {
            format!("unknown source reference: {cell}")
        });
    }
    RowSources {
        references: refs.into_values().collect(),
        unresolved,
        entries,
    }
}

pub(super) fn finish(
    entries: BTreeMap<(String, String), Facts>,
) -> Vec<RequirementSourceInventoryEntry> {
    entries
        .into_iter()
        .map(
            |((requirement_id, source_path), facts)| RequirementSourceInventoryEntry {
                requirement_id,
                source_path,
                provenance: facts.provenance.into_iter().collect(),
                unresolved: facts.unresolved.into_iter().collect(),
            },
        )
        .collect()
}

fn add(entries: &mut BTreeMap<(String, String), Facts>, id: &str, path: &str, provenance: &str) {
    entries
        .entry((id.to_string(), path.to_string()))
        .or_default()
        .provenance
        .insert(provenance.to_string());
}
