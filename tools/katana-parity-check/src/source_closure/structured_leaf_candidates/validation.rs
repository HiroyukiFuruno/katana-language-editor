use std::collections::BTreeMap;

use super::super::artifact_model::{BranchCatalogArtifact, SourceClosureArtifact};
use super::super::fingerprint::sha256_hex;
use super::super::requirement_binding::{RequirementBinding, RequirementBindingResult};
use super::super::source_action_binding::{
    SourceActionBindingReport, SourceActionFact, SourceActionRouteCandidate,
};
use super::StructuredLeafCandidate;

pub(super) fn source_file_index(
    source: &SourceClosureArtifact,
) -> Result<BTreeMap<String, String>, String> {
    let mut files = BTreeMap::new();
    for file in &source.files {
        if files
            .insert(file.path.clone(), file.sha256.clone())
            .is_some()
        {
            return Err(format!("duplicate source file path: {}", file.path));
        }
    }
    Ok(files)
}

pub(super) fn branch_index<'a>(
    catalog: &'a BranchCatalogArtifact,
    source_files: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, BranchFacts<'a>>, String> {
    let mut branches = BTreeMap::new();
    for branch in &catalog.branches {
        if !source_files.contains_key(&branch.file) {
            return Err(format!(
                "branch source file is absent from closure: {}",
                branch.file
            ));
        }
        if branches
            .insert(
                branch.branch_id.clone(),
                BranchFacts {
                    file: &branch.file,
                    start_line: branch.span.start_line,
                    end_line: branch.span.end_line,
                    source_excerpt_sha256: &branch.source_excerpt_sha256,
                },
            )
            .is_some()
        {
            return Err(format!("duplicate branch id: {}", branch.branch_id));
        }
    }
    Ok(branches)
}

pub(super) fn binding_index<'a>(
    requirements: &'a RequirementBindingResult,
    branches: &BTreeMap<String, BranchFacts<'_>>,
    source_files: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, &'a RequirementBinding>, String> {
    let mut bindings = BTreeMap::new();
    for binding in &requirements.bindings {
        let source_sha = source_files.get(&binding.source_path).ok_or_else(|| {
            format!(
                "requirement binding source path is absent from closure: {}",
                binding.source_path
            )
        })?;
        if source_sha != &binding.source_sha256 {
            return Err(format!(
                "requirement binding source SHA mismatch: {}",
                binding.source_path
            ));
        }
        let branch = branches.get(&binding.branch_id).ok_or_else(|| {
            format!(
                "requirement binding branch is absent from catalog: {}",
                binding.branch_id
            )
        })?;
        if branch.file != &binding.source_path
            || branch.start_line != binding.span_start_line
            || branch.end_line != binding.span_end_line
            || branch.source_excerpt_sha256 != &binding.source_excerpt_sha256
        {
            return Err(format!(
                "requirement binding branch facts mismatch: {}",
                binding.branch_id
            ));
        }
        let id = SourceActionBindingReport::binding_id(binding);
        if bindings.insert(id.clone(), binding).is_some() {
            return Err(format!("duplicate requirement binding id: {id}"));
        }
    }
    Ok(bindings)
}

pub(super) fn verify_route_facts(
    route: &SourceActionRouteCandidate,
    source_files: &BTreeMap<String, String>,
) -> Result<(), String> {
    for (kind, fact) in [
        ("definition", &route.definition),
        ("construction", &route.construction),
        ("dispatch", &route.dispatch),
        ("dispatch variant pattern", &route.dispatch_variant_pattern),
    ] {
        SourceActionBindingReport::validate_fact_span(fact)?;
        verify_fact(kind, fact, source_files)?;
    }
    Ok(())
}

pub(super) fn route_identity(route: &SourceActionRouteCandidate) -> Result<String, String> {
    let bytes = serde_json::to_vec(route)
        .map_err(|error| format!("serialize source action route identity: {error}"))?;
    Ok(sha256_hex(&bytes))
}

pub(super) fn candidate(
    requirement_binding: RequirementBinding,
    route: SourceActionRouteCandidate,
) -> Result<StructuredLeafCandidate, String> {
    let bytes = serde_json::to_vec(&CandidateIdentity {
        requirement_binding: &requirement_binding,
        route: &route,
    })
    .map_err(|error| format!("serialize structured leaf candidate identity: {error}"))?;
    Ok(StructuredLeafCandidate {
        identity_fingerprint: sha256_hex(&bytes),
        requirement_binding,
        route,
    })
}

pub(super) fn route_order(
    left: &SourceActionRouteCandidate,
    right: &SourceActionRouteCandidate,
) -> std::cmp::Ordering {
    (
        &left.action_variant,
        &left.construction.path,
        &left.construction.span,
        &left.dispatch.span,
    )
        .cmp(&(
            &right.action_variant,
            &right.construction.path,
            &right.construction.span,
            &right.dispatch.span,
        ))
}

fn verify_fact(
    kind: &str,
    fact: &SourceActionFact,
    source_files: &BTreeMap<String, String>,
) -> Result<(), String> {
    let sha256 = source_files
        .get(&fact.path)
        .ok_or_else(|| format!("{kind} source fact is absent from closure: {}", fact.path))?;
    if sha256 != &fact.sha256 {
        return Err(format!("{kind} source fact SHA mismatch: {}", fact.path));
    }
    Ok(())
}

pub(super) struct BranchFacts<'a> {
    file: &'a String,
    start_line: usize,
    end_line: usize,
    source_excerpt_sha256: &'a String,
}

#[derive(serde::Serialize)]
struct CandidateIdentity<'a> {
    requirement_binding: &'a RequirementBinding,
    route: &'a SourceActionRouteCandidate,
}
