use serde::{Deserialize, Serialize};

use super::artifact_model::SourceClosureArtifact;
use super::source_requirement_alias_ledger::AliasEntry;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RequirementSourceReference {
    pub(crate) raw_reference: String,
    pub(crate) normalized_path: Option<String>,
    pub(crate) status: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RequirementSourceInventoryRow {
    pub(crate) requirement_id: String,
    pub(crate) source_cell: String,
    pub(crate) references: Vec<RequirementSourceReference>,
    pub(crate) unresolved: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RequirementSourceInventoryEntry {
    pub(crate) requirement_id: String,
    pub(crate) source_path: String,
    pub(crate) provenance: Vec<String>,
    pub(crate) unresolved: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RequirementSourceInventoryReport {
    pub(crate) requirements_sha256: String,
    pub(crate) rows: Vec<RequirementSourceInventoryRow>,
    pub(crate) entries: Vec<RequirementSourceInventoryEntry>,
}

impl RequirementSourceInventoryReport {
    pub(crate) fn build(
        requirements: &[u8],
        source: &SourceClosureArtifact,
        aliases: &[AliasEntry],
    ) -> Result<Self, String> {
        super::requirement_source_inventory_build::build(requirements, source, aliases)
    }

    pub(crate) fn entries(&self) -> Vec<AliasEntry> {
        self.entries
            .iter()
            .map(|entry| AliasEntry {
                requirement_id: entry.requirement_id.clone(),
                short_reference: entry
                    .provenance
                    .first()
                    .cloned()
                    .unwrap_or_else(|| entry.source_path.clone()),
                source_path: entry.source_path.clone(),
            })
            .collect()
    }
}
