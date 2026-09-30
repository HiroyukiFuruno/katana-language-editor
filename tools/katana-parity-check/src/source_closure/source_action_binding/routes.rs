use super::super::requirement_binding::RequirementBinding;
use super::super::requirement_binding::RequirementBindingResult;
use super::facts::{binding_id, construction_is_within_branch};
use super::span::parse_span;
use super::types::{
    BRANCH_SPAN_PRECISION, IndexedConstruction, IndexedDefinition, IndexedDispatch,
    SourceActionRouteCandidate, SourceActionUnresolved,
};

#[derive(Default)]
pub(super) struct ConstructionRoutes {
    pub(super) candidates: Vec<SourceActionRouteCandidate>,
    pub(super) unresolved: Vec<SourceActionUnresolved>,
}

pub(super) fn route_construction_is_within_binding(
    route: &SourceActionRouteCandidate,
    binding: &RequirementBinding,
) -> Result<bool, String> {
    let construction = parse_span(&route.construction.span)?;
    Ok(construction.file == binding.source_path
        && construction.start_line >= binding.span_start_line
        && construction.end_line <= binding.span_end_line)
}

impl ConstructionRoutes {
    pub(super) fn build(
        construction: &IndexedConstruction,
        definition: Option<&Result<IndexedDefinition, String>>,
        dispatches: &[IndexedDispatch],
        requirements: &RequirementBindingResult,
    ) -> Self {
        let definition = match definition {
            Some(Ok(definition)) => definition,
            Some(Err(reason)) => {
                return Self::unresolved(construction, "ambiguous_definition", reason);
            }
            None => {
                return Self::unresolved(
                    construction,
                    "missing_definition",
                    "construction has no indexed AppAction definition",
                );
            }
        };
        let matching = dispatches
            .iter()
            .filter(|dispatch| dispatch.action_variant == construction.variant)
            .collect::<Vec<_>>();
        if matching.is_empty() {
            return Self::unresolved(
                construction,
                "missing_dispatch_arm",
                "construction has no explicit dispatch arm for its AppAction variant",
            );
        }
        let mut result = Self::default();
        for dispatch in matching {
            if !dispatch.unresolved_reasons.is_empty() {
                result
                    .unresolved
                    .extend(dispatch.unresolved_reasons.iter().map(|reason| {
                        SourceActionUnresolved {
                            kind: "unresolved_dispatch_arm".into(),
                            source_edge_kind: None,
                            source_edge_detail: None,
                            source_edge_resolution: None,
                            action_variant: Some(construction.variant.clone()),
                            path: dispatch.path.clone(),
                            span: Some(dispatch.span.clone()),
                            reason: reason.clone(),
                        }
                    }));
                continue;
            }
            let requirement_binding_ids = requirements
                .bindings
                .iter()
                .filter(|binding| {
                    binding.source_path == construction.path
                        && construction_is_within_branch(construction, binding)
                })
                .map(binding_id)
                .collect();
            result.candidates.push(SourceActionRouteCandidate {
                action_variant: construction.variant.clone(),
                definition: definition.fact.clone(),
                construction: construction.fact.clone(),
                dispatch: dispatch.fact.clone(),
                dispatch_variant_pattern: dispatch.variant_pattern.clone(),
                requirement_binding_ids,
                branch_span_precision: BRANCH_SPAN_PRECISION.into(),
            });
        }
        result
    }

    fn unresolved(construction: &IndexedConstruction, kind: &str, reason: &str) -> Self {
        Self {
            candidates: Vec::new(),
            unresolved: vec![SourceActionUnresolved {
                kind: kind.into(),
                source_edge_kind: None,
                source_edge_detail: None,
                source_edge_resolution: None,
                action_variant: Some(construction.variant.clone()),
                path: construction.path.clone(),
                span: Some(construction.fact.span.clone()),
                reason: reason.into(),
            }],
        }
    }
}
