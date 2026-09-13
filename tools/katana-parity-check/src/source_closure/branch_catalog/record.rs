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
    profile_ids: &[String],
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
        active_profile_ids: profile_ids.to_vec(),
        inactive_profile_predicates,
        outcomes: vec![],
        incoming_edges: vec![edge.id.clone()],
        source_excerpt_sha256: sha256_hex(condition.as_bytes()),
    })
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
