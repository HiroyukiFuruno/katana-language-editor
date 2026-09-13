use crate::release_gate_sources::{
    JUSTFILE, PUBLISH_CRATES, PUBLISH_CRATES_EVIDENCE, RELEASE_BLOCKER_AUDIT_EVIDENCE,
    RELEASE_BLOCKERS, RELEASE_COMPLETION, RELEASE_COMPLETION_EVIDENCE, RELEASE_GATE_EVIDENCE,
    RELEASE_WORKFLOW, RELEASE_WORKFLOW_EVIDENCE,
};

pub(crate) struct ReleaseGateAudit;

impl ReleaseGateAudit {
    pub(crate) fn validate() -> Result<(), String> {
        for needle in RELEASE_GATE_EVIDENCE {
            if !JUSTFILE.contains(needle) {
                return Err(format!(
                    "Justfile is missing KLE release gate evidence: {needle}"
                ));
            }
        }
        crate::source_closure::source_requirements_ledger::SourceRequirementsLedger::validate_checked_in_coverage()?;
        crate::source_closure::source_requirement_alias_ledger::RequirementSourceAliasLedger::validate_checked_in_coverage()?;
        Self::validate_kle_release_gate()?;
        for needle in RELEASE_WORKFLOW_EVIDENCE {
            if !RELEASE_WORKFLOW.contains(needle) {
                return Err(format!(
                    "Release workflow is missing KLE publication evidence: {needle}"
                ));
            }
        }
        for needle in PUBLISH_CRATES_EVIDENCE {
            if !PUBLISH_CRATES.contains(needle) {
                return Err(format!(
                    "publish-crates.sh is missing crates.io publish visibility evidence: {needle}"
                ));
            }
        }
        for needle in RELEASE_COMPLETION_EVIDENCE {
            if !RELEASE_COMPLETION.contains(needle) {
                return Err(format!(
                    "verify-release-completion.sh is missing release completion audit evidence: {needle}"
                ));
            }
        }
        for needle in RELEASE_BLOCKER_AUDIT_EVIDENCE {
            if !RELEASE_BLOCKERS.contains(needle) {
                return Err(format!(
                    "verify-release-blockers.sh is missing release blocker audit evidence: {needle}"
                ));
            }
        }
        Self::validate_release_check_ordering()?;
        Self::validate_test_and_build_workflow()?;
        Self::validate_release_workflow_ordering()?;
        Self::validate_release_dependency_graph_ordering()?;
        Self::validate_pre_push_gate()?;
        Self::validate_publish_crate_visibility_ordering()?;
        Self::validate_source_closure_native_host_contract()?;
        Self::validate_release_docs_completion_boundary()?;
        Self::validate_completion_audit_is_read_only()
    }

    pub(crate) fn line_index(lines: &[&str], prefix: &str) -> Option<usize> {
        lines.iter().position(|line| line.starts_with(prefix))
    }

    pub(crate) fn exact_line_index(lines: &[&str], needle: &str) -> Option<usize> {
        lines.iter().position(|line| line.trim() == needle)
    }

    pub(crate) fn dependencies_for_recipe<'a>(
        lines: &'a [&'a str],
        recipe_name: &str,
    ) -> Result<Vec<&'a str>, String> {
        let recipe_prefix = format!("{recipe_name}:");
        let index = Self::line_index(lines, recipe_prefix.as_str())
            .ok_or_else(|| format!("Justfile is missing {recipe_name} recipe definition"))?;
        let line = lines[index];
        let mut parts = line.splitn(2, ':');
        parts.next();
        let deps = parts.next().unwrap_or("");
        Ok(deps
            .split_whitespace()
            .filter(|dep| !dep.is_empty())
            .collect())
    }

    pub(crate) fn validate_release_check_ordering() -> Result<(), String> {
        let lines: Vec<&str> = JUSTFILE.lines().collect();
        Self::validate_release_check_ordering_from_lines(&lines)
    }

    pub(crate) fn validate_release_check_ordering_from_lines(lines: &[&str]) -> Result<(), String> {
        let release_check_index = Self::line_index(lines, "release-check:")
            .ok_or_else(|| "Justfile is missing release-check recipe definition".to_string())?;

        let release_check_line = lines[release_check_index];
        if !release_check_line.contains("release-check-clean-generated-artifacts") {
            return Err(
                "release-check must depend on release-check-clean-generated-artifacts".to_string(),
            );
        }

        if release_check_line.contains("release-verify") {
            return Err(
                "release-check must invoke release-verify in its command body, not as a dependency"
                    .to_string(),
            );
        }
        if release_check_line.contains("katana-repo-adapter-check") {
            return Err(
                "release-check must not depend on a downstream KatanA adapter check".to_string(),
            );
        }
        let mut release_verify_invocation_line = None;
        for (i, line) in lines.iter().enumerate().skip(release_check_index + 1) {
            if !line.starts_with("    ") {
                break;
            }
            if line.trim() == "just release-verify" {
                release_verify_invocation_line = Some(i);
            }
            if line.trim().contains("katana-repo-adapter-check") {
                return Err(
                    "release-check must not invoke a downstream KatanA adapter check".to_string(),
                );
            }
        }

        release_verify_invocation_line.ok_or_else(|| {
            "release-check must invoke release-verify in its command body".to_string()
        })?;

        Ok(())
    }
}
