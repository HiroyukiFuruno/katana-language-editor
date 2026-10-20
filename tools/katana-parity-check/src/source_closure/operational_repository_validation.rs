use super::operational_input::{EvidenceRef, RootProvenance};
use super::operational_repository_provenance::is_full_git_revision;
use super::operational_validation::InputVerifier;

pub(super) fn validate_repository_metadata(root: &RootProvenance) -> Result<(), String> {
    for (revision, clean, name) in [
        (&root.kle_revision, root.kle_worktree_clean, "KLE"),
        (&root.kuc_revision, root.kuc_worktree_clean, "KUC"),
    ] {
        if !is_full_git_revision(revision) {
            return Err(format!("{name} revision must be a full Git SHA"));
        }
        if !clean {
            return Err(format!("{name} worktree must be clean"));
        }
    }
    Ok(())
}

impl<'a> InputVerifier<'a> {
    pub(super) fn verify_repository_provenance(
        &mut self,
        revision: &str,
        clean: bool,
        revision_evidence: &EvidenceRef,
        worktree_status_evidence: &EvidenceRef,
        label: &str,
    ) -> Result<(), String> {
        if !clean {
            return Err(format!("{label} worktree must be clean"));
        }
        let revision_bytes = self.read_evidence(revision_evidence, "read-only-local-source")?;
        if String::from_utf8_lossy(&revision_bytes).trim() != revision {
            return Err(format!(
                "{label} revision evidence does not match root revision"
            ));
        }
        let status = self.read_evidence(worktree_status_evidence, "read-only-local-source")?;
        if !status.is_empty() {
            return Err(format!("{label} worktree status evidence is not clean"));
        }
        if revision_evidence.command_or_source != "git rev-parse HEAD" {
            return Err(format!("{label} revision evidence command is invalid"));
        }
        if worktree_status_evidence.command_or_source
            != "git status --porcelain=v1 --untracked-files=all"
        {
            return Err(format!(
                "{label} worktree status evidence command is invalid"
            ));
        }
        Ok(())
    }
}
