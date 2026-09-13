use super::super::fingerprint::sha256_hex;
use super::super::model::{ActionOrigin, BranchRecord, SourceClosureFile};

pub(super) fn identity_fingerprint(branch: &BranchRecord, action: &ActionOrigin) -> String {
    let facts = format!("branch={}\0action={}", branch.branch_id, action.action_id);
    sha256_hex(facts.as_bytes())
}

pub(super) fn candidate_fingerprint(
    identity: &str,
    source_file: &SourceClosureFile,
    branch: &BranchRecord,
    action: &ActionOrigin,
) -> String {
    let facts = format!(
        "identity={}\0source_file={}\0source_sha256={}\0branch={}\0{}\0action={}\0{}",
        identity,
        source_file.path,
        source_file.sha256,
        branch.branch_id,
        branch_facts(branch),
        action.action_id,
        action_facts(action),
    );
    sha256_hex(facts.as_bytes())
}

pub(super) fn provenance(
    source_file: &SourceClosureFile,
    branch: &BranchRecord,
    action: &ActionOrigin,
    candidate_fingerprint: &str,
) -> Vec<String> {
    vec![
        format!("source-file:{}@{}", source_file.path, source_file.sha256),
        format!("source-branch:{}", branch.branch_id),
        format!("action-origin:{}@{}", action.action_id, action.fingerprint),
        format!("candidate-fingerprint:{candidate_fingerprint}"),
    ]
}

fn branch_facts(branch: &BranchRecord) -> String {
    format!(
        "file={}\0symbol={}\0span={}:{}\0kind={}\0condition={}\0profiles={}\0excerpt={}",
        branch.file,
        branch.symbol,
        branch.span.start_line,
        branch.span.end_line,
        branch.kind,
        branch.condition,
        branch.active_profile_ids.join(","),
        branch.source_excerpt_sha256,
    )
}

fn action_facts(action: &ActionOrigin) -> String {
    format!(
        "definition={}\0sites={}\0classification={}\0profiles={}\0routes={}\0kle_leafs={}\0fingerprint={}",
        action.definition_span,
        action.construction_sites.join("|"),
        action.origin_classification,
        action.active_profile_ids.join(","),
        action.forward_route.join("|"),
        action.kle_leafs.join(","),
        action.fingerprint,
    )
}
