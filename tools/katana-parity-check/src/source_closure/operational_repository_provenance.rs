use std::path::Path;

use super::operational_input::{EvidenceRef, FULL_GIT_SHA_LENGTH};
use super::operational_output::capture_bytes;
use super::operational_process::git_output;

pub(super) struct RepositoryRevisionCapture {
    pub(super) revision: String,
    pub(super) revision_evidence: EvidenceRef,
    pub(super) worktree_status_evidence: EvidenceRef,
}

pub(super) fn capture_repository_revision(
    stage_root: &Path,
    repository_root: &Path,
    repository_name: &str,
    run_id: &str,
    evidence: &mut Vec<EvidenceRef>,
) -> Result<RepositoryRevisionCapture, String> {
    let revision_bytes = git_output(repository_root, &["rev-parse", "HEAD"])?;
    let revision = String::from_utf8(revision_bytes.clone())
        .map_err(|error| format!("{repository_name} revision is not UTF-8: {error}"))?
        .trim()
        .to_string();
    if !is_full_git_revision(&revision) {
        return Err(format!("{repository_name} revision is not a full Git SHA"));
    }
    let status = git_output(
        repository_root,
        &["status", "--porcelain=v1", "--untracked-files=all"],
    )?;
    if !status.is_empty() {
        return Err(format!(
            "{repository_name} worktree must be clean for source-closure provenance"
        ));
    }
    let revision_evidence = capture_bytes(
        stage_root,
        &format!("provenance/{repository_name}/revision.txt"),
        &revision_bytes,
        run_id,
        "read-only-local-source",
        "git rev-parse HEAD",
        evidence,
    )?;
    let worktree_status_evidence = capture_bytes(
        stage_root,
        &format!("provenance/{repository_name}/worktree-status.txt"),
        &status,
        run_id,
        "read-only-local-source",
        "git status --porcelain=v1 --untracked-files=all",
        evidence,
    )?;
    Ok(RepositoryRevisionCapture {
        revision,
        revision_evidence,
        worktree_status_evidence,
    })
}

pub(super) fn is_full_git_revision(value: &str) -> bool {
    value.len() == FULL_GIT_SHA_LENGTH && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
