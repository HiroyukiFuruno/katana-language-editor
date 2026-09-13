use std::collections::{BTreeMap, BTreeSet};

use super::artifact_model::SourceClosureArtifact;
use super::requirement_source_inventory::{
    RequirementSourceInventoryReport, RequirementSourceInventoryRow,
};
use super::requirement_source_inventory_scan::{Facts, RowSources};
use super::source_requirement_alias_ledger::{AliasEntry, RequirementSourceAliasLedger};

pub(super) fn build(
    bytes: &[u8],
    source: &SourceClosureArtifact,
    aliases: &[AliasEntry],
) -> Result<RequirementSourceInventoryReport, String> {
    let text = std::str::from_utf8(bytes)
        .map_err(|error| format!("editor requirements input is not UTF-8: {error}"))?;
    let source_rows = RequirementSourceAliasLedger::requirement_source_rows(text)?;
    let ids = source_rows
        .iter()
        .map(|row| row.requirement_id.as_str())
        .collect::<BTreeSet<_>>();
    let indexed = index_aliases(aliases, &ids)?;
    let mut seen = BTreeSet::new();
    let mut rows = Vec::new();
    let mut all: BTreeMap<(String, String), Facts> = BTreeMap::new();
    for row in source_rows {
        if !seen.insert(row.requirement_id.clone()) {
            return Err(format!("duplicate requirement id: {}", row.requirement_id));
        }
        let RowSources {
            references,
            unresolved,
            entries,
        } = super::requirement_source_inventory_scan::row(
            &row.requirement_id,
            &row.source_cell,
            source,
            indexed.get(row.requirement_id.as_str()),
        );
        rows.push(RequirementSourceInventoryRow {
            requirement_id: row.requirement_id,
            source_cell: row.source_cell,
            references,
            unresolved: unresolved.into_iter().collect(),
        });
        for (key, facts) in entries {
            all.entry(key)
                .or_default()
                .provenance
                .extend(facts.provenance);
        }
    }
    Ok(RequirementSourceInventoryReport {
        requirements_sha256: super::fingerprint::sha256_hex(bytes),
        rows,
        entries: super::requirement_source_inventory_scan::finish(all),
    })
}

fn index_aliases<'a>(
    aliases: &'a [AliasEntry],
    ids: &BTreeSet<&str>,
) -> Result<BTreeMap<&'a str, Vec<&'a AliasEntry>>, String> {
    let mut result = BTreeMap::new();
    let mut keys = BTreeSet::new();
    for alias in aliases {
        if !ids.contains(alias.requirement_id.as_str()) {
            return Err(format!(
                "alias has unknown requirement id: {}",
                alias.requirement_id
            ));
        }
        if !keys.insert((alias.requirement_id.as_str(), alias.source_path.as_str())) {
            return Err(format!(
                "duplicate alias source: {}:{}",
                alias.requirement_id, alias.source_path
            ));
        }
        result
            .entry(alias.requirement_id.as_str())
            .or_insert_with(Vec::new)
            .push(alias);
    }
    Ok(result)
}
