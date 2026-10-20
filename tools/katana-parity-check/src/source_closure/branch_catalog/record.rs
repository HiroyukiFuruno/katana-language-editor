use std::fs;
use std::path::Path;

use super::source::{ParsedSpan, branch_condition};
use crate::source_closure::fingerprint::sha256_hex;
use crate::source_closure::model::{
    BranchRecord, ClosureEdge, InactiveProfilePredicate, ProfileRecord, TextSpan,
};

pub(super) fn from_edge(
    edge: &ClosureEdge,
    profiles: &[ProfileRecord],
    katana_root: &Path,
) -> Result<BranchRecord, String> {
    let parsed_span = ParsedSpan::parse(&edge.span)?;
    let source_path = katana_root.join(&edge.from_file);
    let source = fs::read_to_string(&source_path)
        .map_err(|error| format!("failed to read branch source {}: {error}", edge.from_file))?;
    let condition = branch_condition(&edge.kind, edge.to_symbol.as_deref(), &source, &parsed_span)
        .map_err(|error| {
            format!(
                "branch catalog source extraction failed: file={} edge_id={} kind={} span={}: {error}",
                edge.from_file, edge.id, edge.kind, edge.span
            )
        })?;
    let text_span = TextSpan {
        start_line: parsed_span.start_line,
        end_line: parsed_span.end_line,
    };
    let inactive_profile_predicates = inactive_profile_predicates(edge, &parsed_span, profiles);
    let active_profile_ids = active_profile_ids(profiles, &inactive_profile_predicates);
    let id_facts = format!(
        "{}\0{}\0{}\0{}\0{}\0{}",
        edge.from_file, edge.from_symbol, edge.span, edge.kind, condition, edge.id
    );
    let branch_id = format!("branch:{}", sha256_hex(id_facts.as_bytes()));
    Ok(BranchRecord {
        branch_id,
        file: edge.from_file.clone(),
        symbol: edge.from_symbol.clone(),
        span: text_span,
        kind: edge.kind.clone(),
        condition: condition.clone(),
        active_profile_ids,
        inactive_profile_predicates,
        outcomes: vec![],
        incoming_edges: vec![edge.id.clone()],
        source_excerpt_sha256: sha256_hex(condition.as_bytes()),
    })
}

fn active_profile_ids(
    profiles: &[ProfileRecord],
    inactive: &[InactiveProfilePredicate],
) -> Vec<String> {
    let inactive_profile_ids = inactive
        .iter()
        .map(|predicate| predicate.profile_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    profiles
        .iter()
        .map(|profile| profile.id.clone())
        .filter(|profile_id| !inactive_profile_ids.contains(profile_id.as_str()))
        .collect()
}

fn inactive_profile_predicates(
    edge: &ClosureEdge,
    span: &ParsedSpan,
    profiles: &[ProfileRecord],
) -> Vec<InactiveProfilePredicate> {
    if edge.kind != "cfg" {
        return vec![];
    }
    let Some(predicate) = edge.to_symbol.as_deref() else {
        return vec![];
    };
    let cfg_span = format!("{}:{}:{}", edge.from_file, span.start_line, span.end_line);
    let cfg_id = format!(
        "cfg:{}",
        sha256_hex(format!("{}\0{}\0{}", edge.from_file, cfg_span, predicate).as_bytes())
    );
    let mut inactive = profiles
        .iter()
        .flat_map(|profile| {
            profile
                .inactive_cfg_edges
                .iter()
                .filter(|inactive| inactive.edge_id == cfg_id)
                .map(|inactive| InactiveProfilePredicate {
                    profile_id: profile.id.clone(),
                    predicate: inactive.predicate.clone(),
                })
        })
        .collect::<Vec<_>>();
    inactive.sort_by(|left, right| {
        left.profile_id
            .cmp(&right.profile_id)
            .then(left.predicate.cmp(&right.predicate))
    });
    inactive
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excludes_profiles_with_a_captured_inactive_cfg_edge() -> Result<(), String> {
        let edge = ClosureEdge {
            id: "edge:cfg".to_string(),
            from_file: "src/lib.rs".to_string(),
            kind: "cfg".to_string(),
            from_symbol: "gated".to_string(),
            to_path: None,
            to_symbol: Some("target_os = windows".to_string()),
            lexical_resolution: None,
            span: "katana:src/lib.rs:1:0-1:1".to_string(),
        };
        let span = ParsedSpan::parse(&edge.span)?;
        let predicate = edge
            .to_symbol
            .as_deref()
            .ok_or_else(|| "fixture cfg predicate missing".to_string())?;
        let cfg_id = format!(
            "cfg:{}",
            sha256_hex(
                format!(
                    "{}\0{}:{}:{}\0{}",
                    edge.from_file, edge.from_file, span.start_line, span.end_line, predicate,
                )
                .as_bytes(),
            )
        );
        let profiles = ["macos-latest", "windows-latest", "ubuntu-latest"]
            .into_iter()
            .map(|id| ProfileRecord {
                id: id.to_string(),
                runner_label: id.to_string(),
                katana_revision: "revision".to_string(),
                fingerprint: "fingerprint".to_string(),
                rustc_host_triple: "host".to_string(),
                rustc_cfg_sha256: "cfg".to_string(),
                cargo_resolution_sha256: "resolution".to_string(),
                lockfile_sha256: "lock".to_string(),
                active_edge_ids: vec![],
                inactive_cfg_edges: (id == "windows-latest")
                    .then(|| super::super::super::operational_input::InactiveCfgEdge {
                        edge_id: cfg_id.clone(),
                        predicate: "target_os = windows".to_string(),
                        span: edge.span.clone(),
                    })
                    .into_iter()
                    .collect(),
            })
            .collect::<Vec<_>>();
        let inactive = inactive_profile_predicates(&edge, &span, &profiles);

        assert_eq!(
            active_profile_ids(&profiles, &inactive),
            ["macos-latest", "ubuntu-latest"]
        );
        Ok(())
    }
}
