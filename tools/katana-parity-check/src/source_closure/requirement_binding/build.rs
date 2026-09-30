use std::collections::BTreeSet;

use super::super::artifact_model::{BranchCatalogArtifact, SourceClosureArtifact};
use super::super::fingerprint::sha256_hex;
use super::super::operational_input::FIXED_KATANA_REVISION;
use super::super::root_model::ManifestRoot;
use super::super::source_requirement_alias_ledger::{
    ALIAS_LEDGER, AliasEntry, RequirementSourceAliasLedger, SOURCE_UNIVERSE,
};
use super::helpers::{binding, ensure_same_root, unbound_branch};
use super::types::{RequirementBindingResult, RequirementBindingUnresolved};

impl RequirementSourceAliasLedger {
    pub(crate) fn bind_verified_requirements(
        source: &SourceClosureArtifact,
        branches: &BranchCatalogArtifact,
    ) -> Result<RequirementBindingResult, String> {
        ensure_checked_in_root(&source.root)?;
        let entries = Self::verified_alias_entries()?;
        RequirementBindingResult::build(&entries, source, branches)
    }
}

fn ensure_checked_in_root(root: &ManifestRoot) -> Result<(), String> {
    if root.katana_revision != FIXED_KATANA_REVISION {
        return Err("verified requirement binding has a foreign KatanA revision".into());
    }
    if root.requirement_source_aliases_fingerprint != sha256_hex(ALIAS_LEDGER.as_bytes()) {
        return Err("verified requirement binding has a foreign alias ledger fingerprint".into());
    }
    if root.source_universe_fingerprint != sha256_hex(SOURCE_UNIVERSE.as_bytes()) {
        return Err(
            "verified requirement binding has a foreign source universe fingerprint".into(),
        );
    }
    Ok(())
}

impl RequirementBindingResult {
    pub(crate) fn build(
        entries: &[AliasEntry],
        source: &SourceClosureArtifact,
        branches: &BranchCatalogArtifact,
    ) -> Result<Self, String> {
        ensure_same_root(&source.root, &branches.root)?;

        let mut bindings = Vec::new();
        let mut unresolved = Vec::new();
        let mut bound_branch_ids = BTreeSet::new();
        let mut keys = BTreeSet::new();
        let mut alias_keys = BTreeSet::new();
        let mut branch_ids = BTreeSet::new();

        for branch in &branches.branches {
            if !branch_ids.insert(branch.branch_id.as_str()) {
                return Err(format!("duplicate branch id: {}", branch.branch_id));
            }
        }

        for entry in entries {
            if !alias_keys.insert((entry.requirement_id.as_str(), entry.source_path.as_str())) {
                return Err(format!(
                    "duplicate requirement source key: {}:{}",
                    entry.requirement_id, entry.source_path
                ));
            }
            let matching_files = source
                .files
                .iter()
                .filter(|file| file.path == entry.source_path)
                .collect::<Vec<_>>();
            if matching_files.len() != 1 {
                let reason = match matching_files.len() {
                    0 => "source file not found".to_string(),
                    count => format!("source file is duplicated ({count} records)"),
                };
                unresolved.push(RequirementBindingUnresolved {
                    requirement_id: entry.requirement_id.clone(),
                    short_reference: entry.short_reference.clone(),
                    source_path: entry.source_path.clone(),
                    reason,
                });
                continue;
            }
            let source_file = matching_files[0];
            let matching_branches = branches
                .branches
                .iter()
                .filter(|branch| branch.file == entry.source_path)
                .collect::<Vec<_>>();
            if matching_branches.is_empty() {
                unresolved.push(RequirementBindingUnresolved {
                    requirement_id: entry.requirement_id.clone(),
                    short_reference: entry.short_reference.clone(),
                    source_path: entry.source_path.clone(),
                    reason: "no branch found in source file".to_string(),
                });
                continue;
            }
            for branch in matching_branches {
                let key = (
                    entry.requirement_id.as_str(),
                    entry.source_path.as_str(),
                    branch.branch_id.as_str(),
                );
                if !keys.insert(key) {
                    return Err(format!(
                        "duplicate requirement binding key: {}:{}:{}",
                        entry.requirement_id, entry.source_path, branch.branch_id
                    ));
                }
                bound_branch_ids.insert(branch.branch_id.clone());
                bindings.push(binding(entry, source_file.sha256.as_str(), branch));
            }
        }

        bindings.sort_by(|left, right| {
            (&left.requirement_id, &left.source_path, &left.branch_id).cmp(&(
                &right.requirement_id,
                &right.source_path,
                &right.branch_id,
            ))
        });
        unresolved.sort_by(|left, right| {
            (
                &left.requirement_id,
                &left.source_path,
                &left.short_reference,
                &left.reason,
            )
                .cmp(&(
                    &right.requirement_id,
                    &right.source_path,
                    &right.short_reference,
                    &right.reason,
                ))
        });
        let mut unbound_branches = branches
            .branches
            .iter()
            .filter(|branch| !bound_branch_ids.contains(&branch.branch_id))
            .map(unbound_branch)
            .collect::<Vec<_>>();
        unbound_branches.sort_by(|left, right| left.branch_id.cmp(&right.branch_id));

        let mut result = RequirementBindingResult {
            root: source.root.clone(),
            bindings,
            unresolved,
            unbound_branches,
            fingerprint: String::new(),
        };
        result.fingerprint = result
            .fingerprint_value()
            .map_err(|error| format!("serialize requirement binding fingerprint: {error}"))?;
        Ok(result)
    }
}
