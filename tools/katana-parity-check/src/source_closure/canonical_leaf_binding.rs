use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::artifact_model::{BranchCatalogArtifact, LeafStatusRecord, SourceClosureArtifact};
use super::requirement_binding::RequirementBindingResult;
use super::requirement_source_inventory::RequirementSourceInventoryReport;
use super::scan_state::ScanState;
use super::source_action_binding::SourceActionBindingReport;
use super::source_requirement_alias_ledger::{REQUIREMENTS, RequirementSourceAliasLedger};
use super::structured_leaf_candidates::StructuredLeafCandidatesReport;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CanonicalLeafBindingEvidence {
    pub(crate) inventory: RequirementSourceInventoryReport,
    pub(crate) t2: RequirementBindingResult,
    pub(crate) t3: SourceActionBindingReport,
    pub(crate) structured: StructuredLeafCandidatesReport,
}

impl CanonicalLeafBindingEvidence {
    pub(crate) fn build(
        source: &SourceClosureArtifact,
        branches: &BranchCatalogArtifact,
        state: &ScanState,
    ) -> Result<Self, String> {
        let inventory = inventory(source)?;
        let t2 = RequirementBindingResult::build(&inventory.entries(), source, branches)?;
        let t3 = SourceActionBindingReport::build(state, source, &t2)?;
        let structured = StructuredLeafCandidatesReport::build(source, branches, &t2, &t3)?;
        Ok(Self {
            inventory,
            t2,
            t3,
            structured,
        })
    }

    pub(crate) fn verify_runtime_requirements() -> Result<(), String> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/v0-1-0-editor-requirements.md");
        let bytes = std::fs::read(&path)
            .map_err(|error| format!("failed to read checked-in requirements: {error}"))?;
        if bytes == REQUIREMENTS.as_bytes() {
            Ok(())
        } else {
            Err("checked-in requirements differ from checker input".into())
        }
    }

    pub(crate) fn validate(
        &self,
        source: &SourceClosureArtifact,
        branches: &BranchCatalogArtifact,
    ) -> Result<(), String> {
        let expected_inventory = inventory(source)?;
        same("inventory", &expected_inventory, &self.inventory)?;
        let expected_t2 =
            RequirementBindingResult::build(&self.inventory.entries(), source, branches)?;
        same("T2 requirement binding", &expected_t2, &self.t2)?;
        self.t2.validate_fingerprint()?;
        RequirementBindingResult::ensure_same_root(&source.root, &self.t3.root)?;
        if self.t3.requirement_binding_fingerprint != self.t2.fingerprint {
            return Err("T3 requirement binding parent fingerprint mismatch".into());
        }
        self.t3.validate_fingerprint()?;
        self.t3.validate_branch_span_precision()?;
        let expected_structured =
            StructuredLeafCandidatesReport::build(source, branches, &self.t2, &self.t3)?;
        same(
            "structured leaf candidates",
            &expected_structured,
            &self.structured,
        )
    }

    pub(crate) fn project_leafs(
        &self,
        branches: &BranchCatalogArtifact,
    ) -> Result<Vec<LeafStatusRecord>, String> {
        super::canonical_leaf_projection::project(self, branches)
    }

    pub(crate) fn validate_projection(
        &self,
        branches: &BranchCatalogArtifact,
        leafs: &[LeafStatusRecord],
    ) -> Result<(), String> {
        super::canonical_leaf_projection::validate(self, branches, leafs)
    }

    pub(crate) fn release_completeness_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self
            .inventory
            .rows
            .iter()
            .any(|row| !row.unresolved.is_empty())
        {
            errors.push("canonical binding has unresolved inventory rows".into());
        }
        if !self.t2.unresolved.is_empty() || !self.t2.unbound_branches.is_empty() {
            errors.push("canonical binding has unresolved or unbound T2 population".into());
        }
        if !self.t3.unresolved.is_empty() || !self.t3.unmatched_requirement_binding_ids.is_empty() {
            errors.push("canonical binding has unresolved or unmatched T3 population".into());
        }
        if !self.structured.unmatched_requirement_binding_ids.is_empty()
            || !self.structured.zero_reference_routes.is_empty()
            || !self.structured.requirement_unresolved.is_empty()
            || !self.structured.unbound_branches.is_empty()
            || !self.structured.source_action_unresolved.is_empty()
        {
            errors.push("canonical binding has unresolved structured candidate population".into());
        }
        errors
    }
}

fn inventory(source: &SourceClosureArtifact) -> Result<RequirementSourceInventoryReport, String> {
    let aliases = RequirementSourceAliasLedger::verified_alias_entries()?;
    RequirementSourceInventoryReport::build(REQUIREMENTS.as_bytes(), source, &aliases)
}

fn same<T: Serialize>(name: &str, expected: &T, actual: &T) -> Result<(), String> {
    let expected = serde_json::to_vec(expected)
        .map_err(|error| format!("serialize expected {name}: {error}"))?;
    let actual =
        serde_json::to_vec(actual).map_err(|error| format!("serialize stored {name}: {error}"))?;
    if expected == actual {
        Ok(())
    } else {
        Err(format!("{name} differs from canonical reconstruction"))
    }
}
