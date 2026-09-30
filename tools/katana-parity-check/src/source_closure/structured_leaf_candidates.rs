use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::artifact_model::{BranchCatalogArtifact, SourceClosureArtifact};
use super::fingerprint::sha256_hex;
use super::requirement_binding::{
    RequirementBinding, RequirementBindingResult, RequirementBindingUnresolved, UnboundBranch,
};
use super::source_action_binding::{
    SourceActionBindingReport, SourceActionRouteCandidate, SourceActionUnresolved,
};

mod validation;

use validation::{
    binding_index, branch_index, candidate, route_identity, route_order, source_file_index,
    verify_route_facts,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StructuredLeafCandidate {
    pub(crate) identity_fingerprint: String,
    pub(crate) requirement_binding: RequirementBinding,
    pub(crate) route: SourceActionRouteCandidate,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StructuredLeafCandidatesReport {
    pub(crate) root: super::root_model::ManifestRoot,
    pub(crate) requirement_binding_fingerprint: String,
    pub(crate) source_action_binding_fingerprint: String,
    pub(crate) candidates: Vec<StructuredLeafCandidate>,
    pub(crate) unmatched_requirement_binding_ids: Vec<String>,
    pub(crate) zero_reference_routes: Vec<SourceActionRouteCandidate>,
    pub(crate) requirement_unresolved: Vec<RequirementBindingUnresolved>,
    pub(crate) unbound_branches: Vec<UnboundBranch>,
    pub(crate) source_action_unresolved: Vec<SourceActionUnresolved>,
    pub(crate) fingerprint: String,
}

impl StructuredLeafCandidatesReport {
    pub(crate) fn build(
        source: &SourceClosureArtifact,
        branches: &BranchCatalogArtifact,
        requirements: &RequirementBindingResult,
        actions: &SourceActionBindingReport,
    ) -> Result<Self, String> {
        RequirementBindingResult::ensure_same_root(&source.root, &branches.root)?;
        RequirementBindingResult::ensure_same_root(&source.root, &requirements.root)?;
        RequirementBindingResult::ensure_same_root(&source.root, &actions.root)?;
        requirements.validate_fingerprint()?;
        actions.validate_fingerprint()?;
        actions.validate_branch_span_precision()?;
        if actions.requirement_binding_fingerprint != requirements.fingerprint {
            return Err("source action binding parent fingerprint mismatch".into());
        }

        let source_files = source_file_index(source)?;
        let branches = branch_index(branches, &source_files)?;
        let bindings = binding_index(requirements, &branches, &source_files)?;
        let mut candidates = Vec::new();
        let mut referenced = BTreeSet::new();
        let mut zero_reference_routes = Vec::new();
        let mut route_ids = BTreeSet::new();
        let mut candidate_ids = BTreeSet::new();

        for route in &actions.candidates {
            verify_route_facts(route, &source_files)?;
            if !route_ids.insert(route_identity(route)?) {
                return Err("duplicate source action route candidate".into());
            }
            if route.requirement_binding_ids.is_empty() {
                zero_reference_routes.push(route.clone());
                continue;
            }
            let mut route_references = BTreeSet::new();
            for id in &route.requirement_binding_ids {
                if !route_references.insert(id.as_str()) {
                    return Err(format!("duplicate requirement binding reference: {id}"));
                }
                let binding = bindings
                    .get(id)
                    .ok_or_else(|| format!("unknown requirement binding reference: {id}"))?;
                if !SourceActionBindingReport::route_construction_is_within_binding(route, binding)?
                {
                    return Err(format!(
                        "route construction is outside requirement binding branch span: {id}"
                    ));
                }
                referenced.insert(id.clone());
                let candidate = candidate((*binding).clone(), route.clone())?;
                if !candidate_ids.insert(candidate.identity_fingerprint.clone()) {
                    return Err(format!(
                        "duplicate structured leaf candidate identity: {}",
                        candidate.identity_fingerprint
                    ));
                }
                candidates.push(candidate);
            }
        }
        candidates.sort_by(|left, right| {
            (
                &left.identity_fingerprint,
                &left.requirement_binding.requirement_id,
            )
                .cmp(&(
                    &right.identity_fingerprint,
                    &right.requirement_binding.requirement_id,
                ))
        });
        zero_reference_routes.sort_by(route_order);

        let mut unmatched_requirement_binding_ids = requirements
            .bindings
            .iter()
            .map(SourceActionBindingReport::binding_id)
            .filter(|id| !referenced.contains(id))
            .collect::<Vec<_>>();
        unmatched_requirement_binding_ids.sort();
        if actions.unmatched_requirement_binding_ids != unmatched_requirement_binding_ids {
            return Err("source action binding unmatched requirement references differ".into());
        }

        let mut report = Self {
            root: source.root.clone(),
            requirement_binding_fingerprint: requirements.fingerprint.clone(),
            source_action_binding_fingerprint: actions.fingerprint.clone(),
            candidates,
            unmatched_requirement_binding_ids,
            zero_reference_routes,
            requirement_unresolved: requirements.unresolved.clone(),
            unbound_branches: requirements.unbound_branches.clone(),
            source_action_unresolved: actions.unresolved.clone(),
            fingerprint: String::new(),
        };
        report.fingerprint = report.fingerprint_value()?;
        Ok(report)
    }

    fn fingerprint_value(&self) -> Result<String, String> {
        let input = FingerprintInput {
            root: &self.root,
            requirement_binding_fingerprint: &self.requirement_binding_fingerprint,
            source_action_binding_fingerprint: &self.source_action_binding_fingerprint,
            candidates: &self.candidates,
            unmatched_requirement_binding_ids: &self.unmatched_requirement_binding_ids,
            zero_reference_routes: &self.zero_reference_routes,
            requirement_unresolved: &self.requirement_unresolved,
            unbound_branches: &self.unbound_branches,
            source_action_unresolved: &self.source_action_unresolved,
        };
        let bytes = serde_json::to_vec(&input).map_err(|error| {
            format!("serialize structured leaf candidates fingerprint: {error}")
        })?;
        Ok(sha256_hex(&bytes))
    }
}

#[derive(Serialize)]
struct FingerprintInput<'a> {
    root: &'a super::root_model::ManifestRoot,
    requirement_binding_fingerprint: &'a str,
    source_action_binding_fingerprint: &'a str,
    candidates: &'a [StructuredLeafCandidate],
    unmatched_requirement_binding_ids: &'a [String],
    zero_reference_routes: &'a [SourceActionRouteCandidate],
    requirement_unresolved: &'a [RequirementBindingUnresolved],
    unbound_branches: &'a [UnboundBranch],
    source_action_unresolved: &'a [SourceActionUnresolved],
}

#[cfg(test)]
#[path = "structured_leaf_candidates_tests.rs"]
mod tests;
