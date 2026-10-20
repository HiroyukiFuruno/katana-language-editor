use super::artifact_validator::ArtifactValidationMode;
use super::model::{KATANA_ADOPTION_ISSUE_URL, KatanaHostE2eState, LeafStatusRecord};
use super::validator_helpers::is_placeholder;

pub(super) fn validate(
    leaf: &LeafStatusRecord,
    mode: ArtifactValidationMode,
    errors: &mut Vec<String>,
) {
    if leaf.host_e2e.adoption_issue_url != KATANA_ADOPTION_ISSUE_URL {
        errors.push(format!(
            "leaf-manifest leaf {} must bind host adoption to {}",
            leaf.leaf_id, KATANA_ADOPTION_ISSUE_URL
        ));
    }
    match mode {
        ArtifactValidationMode::Rc | ArtifactValidationMode::KleRelease => {
            validate_pre_publication_boundary(leaf, errors)
        }
        ArtifactValidationMode::Full => validate_full(leaf, errors),
    }
}

fn validate_pre_publication_boundary(leaf: &LeafStatusRecord, errors: &mut Vec<String>) {
    if leaf.host_e2e.state != KatanaHostE2eState::DownstreamRequired || leaf.execution_id.is_some()
    {
        errors.push(format!(
            "pre-publication leaf {} must declare downstream_required host E2E without an execution id",
            leaf.leaf_id
        ));
    }
}

fn validate_full(leaf: &LeafStatusRecord, errors: &mut Vec<String>) {
    let execution_id = leaf.execution_id.as_deref().unwrap_or_default();
    if leaf.host_e2e.state != KatanaHostE2eState::Passing
        || execution_id.trim().is_empty()
        || execution_id.trim_start().starts_with("unresolved:")
        || is_placeholder(execution_id)
    {
        errors.push(format!(
            "full-parity leaf {} requires a passing KatanA host E2E execution id",
            leaf.leaf_id
        ));
    }
}
