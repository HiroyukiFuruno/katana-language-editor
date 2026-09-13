use std::collections::BTreeMap;

use super::super::model::{ActionOrigin, ActionOriginsArtifact, ManifestRoot};
use super::super::scan_state::{ActionConstruction, ActionDefinition, ScanState};
use super::{facts, routes};

type ConstructionIndexes = (
    BTreeMap<String, Vec<String>>,
    BTreeMap<String, Vec<String>>,
    Vec<String>,
);

pub(super) fn materialize(root: ManifestRoot, state: &ScanState) -> ActionOriginsArtifact {
    let definitions = sorted_definitions(state);
    let constructions = sorted_constructions(state);
    let definitions_by_action = index_definitions(definitions);
    let (construction_sites, input_candidate_routes, mut unresolved) =
        index_constructions(&constructions, &definitions_by_action);
    let actions = build_actions(
        &definitions_by_action,
        construction_sites,
        input_candidate_routes,
        state,
        &mut unresolved,
    );
    unresolved.extend(
        state
            .dispatch_fallthroughs
            .iter()
            .map(facts::dispatch_fallthrough_fact),
    );
    unresolved.sort();
    unresolved.dedup();
    ActionOriginsArtifact {
        root,
        actions,
        unresolved_action_origins: unresolved,
    }
}

fn sorted_definitions(state: &ScanState) -> Vec<ActionDefinition> {
    let mut definitions = state.action_definitions.clone();
    definitions.sort_by(facts::definition_order);
    definitions
}

fn sorted_constructions(state: &ScanState) -> Vec<ActionConstruction> {
    let mut constructions = state.action_constructions.clone();
    constructions.sort_by(facts::construction_order);
    constructions
}

fn index_definitions(
    definitions: Vec<ActionDefinition>,
) -> BTreeMap<String, Vec<ActionDefinition>> {
    let mut indexed = BTreeMap::new();
    for definition in definitions {
        indexed
            .entry(facts::action_id(&definition.variant))
            .or_insert_with(Vec::new)
            .push(definition);
    }
    indexed
}

fn index_constructions(
    constructions: &[ActionConstruction],
    definitions: &BTreeMap<String, Vec<ActionDefinition>>,
) -> ConstructionIndexes {
    let mut sites = BTreeMap::new();
    let mut routes = BTreeMap::new();
    let mut unresolved = Vec::new();
    for construction in constructions {
        let Some(variant) = construction.variant.as_deref() else {
            unresolved.extend(facts::unresolved_input_candidate_facts(
                construction,
                "construction does not identify an AppAction variant",
            ));
            unresolved.push(facts::unresolved_site(
                construction,
                "construction does not identify an AppAction variant",
            ));
            continue;
        };
        let id = facts::action_id(variant);
        let definition_count = definitions.get(&id).map_or(0, Vec::len);
        let reason = construction_reason(construction, definition_count);
        if let Some(reason) = reason {
            unresolved.extend(facts::unresolved_input_candidate_facts(
                construction,
                reason,
            ));
            unresolved.push(facts::unresolved_site(construction, reason));
            continue;
        }
        sites
            .entry(id.clone())
            .or_insert_with(Vec::new)
            .push(facts::site_text(construction));
        routes
            .entry(id)
            .or_insert_with(Vec::new)
            .extend(facts::input_candidate_facts(construction));
    }
    (sites, routes, unresolved)
}

fn construction_reason(construction: &ActionConstruction, definition_count: usize) -> Option<&str> {
    if construction.enum_name != "AppAction" {
        return Some(
            construction
                .unresolved_reason
                .as_deref()
                .unwrap_or("construction enum path is not AppAction"),
        );
    }
    if let Some(reason) = construction.unresolved_reason.as_deref() {
        return Some(reason);
    }
    match definition_count {
        0 => Some("direct construction has no indexed AppAction variant definition"),
        1 => None,
        _ => Some("direct construction has ambiguous indexed AppAction variant definitions"),
    }
}

fn build_actions(
    definitions: &BTreeMap<String, Vec<ActionDefinition>>,
    mut sites: BTreeMap<String, Vec<String>>,
    mut candidate_routes: BTreeMap<String, Vec<String>>,
    state: &ScanState,
    unresolved: &mut Vec<String>,
) -> Vec<ActionOrigin> {
    let mut actions = Vec::new();
    for (id, definitions) in definitions {
        if definitions.len() != 1 {
            unresolved.push(format!(
                "{id}: ambiguous indexed AppAction variant definitions at {}",
                definitions
                    .iter()
                    .map(facts::definition_text)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            continue;
        }
        let definition = &definitions[0];
        let construction_sites = sites.remove(id).unwrap_or_default();
        let mut forward_route = state
            .dispatch_arms
            .iter()
            .filter(|arm| format!("AppAction::{}", arm.action_variant) == *id)
            .map(|arm| routes::dispatch_route_fact(arm, state))
            .collect::<Vec<_>>();
        forward_route.sort();
        if let Some(mut candidates) = candidate_routes.remove(id) {
            candidates.sort();
            forward_route.extend(candidates);
        }
        unresolved.extend(
            forward_route
                .iter()
                .filter_map(facts::unresolved_route_fact),
        );
        unresolved.extend(
            forward_route
                .iter()
                .filter(|route| {
                    route.starts_with("input_origin_candidate")
                        && !route.contains("unresolved=none")
                })
                .cloned(),
        );
        unresolved.push(format!(
            "{id}: action origin, dispatch, handler, and final host effect are unproven; definition {}; constructions [{}]",
            facts::definition_text(definition),
            construction_sites.join(", ")
        ));
        actions.push(ActionOrigin {
            action_id: id.clone(),
            definition_span: facts::definition_text(definition),
            construction_sites,
            origin_classification: "unresolved".to_string(),
            active_profile_ids: vec![],
            origin_input: None,
            forward_route,
            kle_leafs: vec![],
            fingerprint: String::new(),
            predecessor_actions: None,
            state_span: None,
            no_renderer_or_shortcut_rationale: None,
        });
    }
    actions.sort_by(|left, right| left.action_id.cmp(&right.action_id));
    actions
}
