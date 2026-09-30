use std::collections::BTreeMap;

use super::super::model::ManifestRoot;
use super::super::scan_state::ActionDefinition;
use super::types::{SourceActionRouteCandidate, SourceActionUnresolved};

pub(super) fn unresolved_definitions(
    definitions: &[ActionDefinition],
    source_files: &BTreeMap<String, String>,
) -> Vec<SourceActionUnresolved> {
    let mut by_variant = BTreeMap::<&str, usize>::new();
    for definition in definitions {
        *by_variant.entry(&definition.variant).or_default() += 1;
    }
    definitions
        .iter()
        .filter(|definition| by_variant[definition.variant.as_str()] > 1)
        .map(|definition| SourceActionUnresolved {
            kind: "ambiguous_definition".into(),
            source_edge_kind: None,
            source_edge_detail: None,
            source_edge_resolution: None,
            action_variant: Some(definition.variant.clone()),
            path: definition.file.clone(),
            span: Some(definition.variant_span.clone()),
            reason: format!(
                "{} AppAction definitions share variant `{}`",
                by_variant[definition.variant.as_str()],
                definition.variant
            ),
        })
        .chain(definitions.iter().filter_map(|definition| {
            if source_files.contains_key(&definition.file) {
                None
            } else {
                Some(SourceActionUnresolved {
                    kind: "definition_source_missing".into(),
                    source_edge_kind: None,
                    source_edge_detail: None,
                    source_edge_resolution: None,
                    action_variant: Some(definition.variant.clone()),
                    path: definition.file.clone(),
                    span: Some(definition.variant_span.clone()),
                    reason: "definition file is absent from closure".into(),
                })
            }
        }))
        .collect()
}

pub(super) fn candidate_order(
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

pub(super) fn unresolved_order(
    left: &SourceActionUnresolved,
    right: &SourceActionUnresolved,
) -> std::cmp::Ordering {
    (
        &left.kind,
        &left.source_edge_kind,
        &left.source_edge_detail,
        &left.source_edge_resolution,
        &left.path,
        &left.action_variant,
        &left.span,
        &left.reason,
    )
        .cmp(&(
            &right.kind,
            &right.source_edge_kind,
            &right.source_edge_detail,
            &right.source_edge_resolution,
            &right.path,
            &right.action_variant,
            &right.span,
            &right.reason,
        ))
}

pub(super) fn ensure_same_root(
    source: &ManifestRoot,
    requirements: &ManifestRoot,
) -> Result<(), String> {
    let source_bytes =
        serde_json::to_vec(source).map_err(|error| format!("serialize source root: {error}"))?;
    let requirement_bytes = serde_json::to_vec(requirements)
        .map_err(|error| format!("serialize requirement root: {error}"))?;
    if source_bytes != requirement_bytes {
        return Err("source action binding roots differ".into());
    }
    Ok(())
}
