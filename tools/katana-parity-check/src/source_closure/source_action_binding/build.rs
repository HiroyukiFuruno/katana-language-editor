use std::collections::BTreeSet;

use super::super::artifact_model::SourceClosureArtifact;
use super::super::fingerprint::sha256_hex;
use super::super::requirement_binding::RequirementBindingResult;
use super::super::scan_state::ScanState;
use super::facts::{
    binding_id, construction_facts, dispatch_fact, index_definitions, source_file_index,
    unresolved_construction, verify_requirement_bindings,
};
use super::order::{candidate_order, ensure_same_root, unresolved_definitions, unresolved_order};
use super::routes::ConstructionRoutes;
use super::types::{
    BRANCH_SPAN_PRECISION, FingerprintInput, SourceActionBindingReport, SourceActionUnresolved,
};

impl SourceActionBindingReport {
    pub(crate) fn build(
        state: &ScanState,
        source: &SourceClosureArtifact,
        requirements: &RequirementBindingResult,
    ) -> Result<Self, String> {
        ensure_same_root(&source.root, &requirements.root)?;
        let source_files = source_file_index(source)?;
        verify_requirement_bindings(requirements, &source_files)?;
        let mut unresolved = Vec::new();
        let definitions = index_definitions(&state.action_definitions, &source_files);
        let mut constructions = Vec::new();
        for construction in &state.action_constructions {
            match construction_facts(construction, &definitions, &source_files) {
                Ok(fact) => constructions.push(fact),
                Err(reason) => unresolved.push(unresolved_construction(construction, &reason)),
            }
        }
        let mut dispatches = Vec::new();
        for arm in &state.dispatch_arms {
            match dispatch_fact(arm, &source_files) {
                Ok(fact) => dispatches.push(fact),
                Err(reason) => unresolved.push(SourceActionUnresolved {
                    kind: "unresolved_dispatch_arm".into(),
                    source_edge_kind: None,
                    source_edge_detail: None,
                    source_edge_resolution: None,
                    action_variant: Some(arm.action_variant.clone()),
                    path: arm.file.clone(),
                    span: Some(arm.arm_span.clone()),
                    reason,
                }),
            }
        }

        let mut candidates = Vec::new();
        let mut matched_requirement_ids = BTreeSet::new();
        let constructed_variants = constructions
            .iter()
            .map(|construction| construction.variant.clone())
            .collect::<BTreeSet<_>>();
        for construction in constructions {
            let routes = ConstructionRoutes::build(
                &construction,
                definitions.get(&construction.variant),
                &dispatches,
                requirements,
            );
            matched_requirement_ids.extend(
                routes
                    .candidates
                    .iter()
                    .flat_map(|candidate| candidate.requirement_binding_ids.iter().cloned()),
            );
            candidates.extend(routes.candidates);
            unresolved.extend(routes.unresolved);
        }

        unresolved.extend(unresolved_definitions(
            &state.action_definitions,
            &source_files,
        ));
        unresolved.extend(
            dispatches
                .iter()
                .filter(|dispatch| !constructed_variants.contains(&dispatch.action_variant))
                .map(|dispatch| SourceActionUnresolved {
                    kind: "dispatch_without_construction".into(),
                    source_edge_kind: None,
                    source_edge_detail: None,
                    source_edge_resolution: None,
                    action_variant: Some(dispatch.action_variant.clone()),
                    path: dispatch.path.clone(),
                    span: Some(dispatch.span.clone()),
                    reason: "explicit dispatch arm has no construction fact".into(),
                }),
        );
        unresolved.extend(state.dispatch_fallthroughs.iter().map(|fallthrough| {
            SourceActionUnresolved {
                kind: "dispatch_fallthrough".into(),
                source_edge_kind: None,
                source_edge_detail: None,
                source_edge_resolution: None,
                action_variant: None,
                path: fallthrough.file.clone(),
                span: Some(fallthrough.arm_span.clone()),
                reason: "dispatch arm does not identify an AppAction variant".into(),
            }
        }));
        for edge in &state.unresolved_edges {
            if edge.kind.trim().is_empty() {
                return Err("unresolved scan edge kind is empty".into());
            }
            if edge
                .to_symbol
                .as_deref()
                .is_some_and(|detail| detail.trim().is_empty())
            {
                return Err("unresolved scan edge detail is empty".into());
            }
            unresolved.push(SourceActionUnresolved {
                kind: "unresolved_scan_edge".into(),
                source_edge_kind: Some(edge.kind.clone()),
                source_edge_detail: edge.to_symbol.clone(),
                source_edge_resolution: edge.lexical_resolution.clone(),
                action_variant: None,
                path: edge.from_file.clone(),
                span: Some(edge.span.clone()),
                reason: format!("unresolved source edge `{}`", edge.kind),
            });
        }
        unresolved.extend(definitions.iter().filter_map(
            |(variant, definition)| match definition {
                Ok(definition) if !constructed_variants.contains(variant) => {
                    Some(SourceActionUnresolved {
                        kind: "definition_without_construction".into(),
                        source_edge_kind: None,
                        source_edge_detail: None,
                        source_edge_resolution: None,
                        action_variant: Some(variant.clone()),
                        path: definition.fact.path.clone(),
                        span: Some(definition.fact.span.clone()),
                        reason: "unique AppAction definition has no construction fact".into(),
                    })
                }
                _ => None,
            },
        ));
        let mut unmatched_requirement_binding_ids = requirements
            .bindings
            .iter()
            .map(binding_id)
            .filter(|id| !matched_requirement_ids.contains(id))
            .collect::<Vec<_>>();
        unmatched_requirement_binding_ids.sort();
        unresolved.extend(
            requirements
                .bindings
                .iter()
                .filter(|binding| {
                    unmatched_requirement_binding_ids
                        .binary_search(&binding_id(binding))
                        .is_ok()
                })
                .map(|binding| SourceActionUnresolved {
                    kind: "requirement_without_action_route".into(),
                    source_edge_kind: None,
                    source_edge_detail: None,
                    source_edge_resolution: None,
                    action_variant: None,
                    path: binding.source_path.clone(),
                    span: Some(format!(
                        "{}:{}-{}",
                        binding.source_path, binding.span_start_line, binding.span_end_line
                    )),
                    reason: "requirement binding has no joined source action route".into(),
                }),
        );
        candidates.sort_by(candidate_order);
        unresolved.sort_by(unresolved_order);

        let digest = FingerprintInput {
            root: &source.root,
            requirement_binding_fingerprint: &requirements.fingerprint,
            branch_span_precision: BRANCH_SPAN_PRECISION,
            candidates: &candidates,
            unresolved: &unresolved,
            unmatched_requirement_binding_ids: &unmatched_requirement_binding_ids,
        };
        let bytes = serde_json::to_vec(&digest)
            .map_err(|error| format!("serialize source action binding fingerprint: {error}"))?;
        Ok(Self {
            root: source.root.clone(),
            requirement_binding_fingerprint: requirements.fingerprint.clone(),
            branch_span_precision: BRANCH_SPAN_PRECISION.into(),
            candidates,
            unresolved,
            unmatched_requirement_binding_ids,
            fingerprint: sha256_hex(&bytes),
        })
    }
}
