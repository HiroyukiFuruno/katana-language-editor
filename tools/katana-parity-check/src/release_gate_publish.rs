use crate::release_gate::ReleaseGateAudit;
use crate::release_gate_sources::{
    PUBLISH_CRATES, RELEASE_BLOCKERS, RELEASE_COMPLETION, RELEASE_DOC, RELEASE_DOC_EVIDENCE,
    RELEASE_READINESS_DOC, RELEASE_READINESS_EVIDENCE,
};

impl ReleaseGateAudit {
    pub(crate) fn validate_publish_crate_visibility_ordering() -> Result<(), String> {
        Self::validate_publish_crate_visibility_from_lines(
            &PUBLISH_CRATES.lines().collect::<Vec<_>>(),
        )
    }

    pub(crate) fn validate_publish_crate_visibility_from_lines(
        lines: &[&str],
    ) -> Result<(), String> {
        Self::validate_kuc_release_set_gate_from_lines(lines)?;
        let publish_core_line =
            Self::line_index(lines, "publish_if_needed katana-language-editor ").ok_or_else(
                || "publish-crates.sh must publish katana-language-editor".to_string(),
            )?;
        let wait_core_line = Self::exact_line_index(lines, "wait_for_crate katana-language-editor")
            .ok_or_else(|| {
                "publish-crates.sh must wait for katana-language-editor visibility".to_string()
            })?;
        let publish_egui_line =
            Self::line_index(lines, "publish_if_needed katana-language-editor-egui ").ok_or_else(
                || "publish-crates.sh must publish katana-language-editor-egui".to_string(),
            )?;
        let wait_egui_line =
            Self::exact_line_index(lines, "wait_for_crate katana-language-editor-egui")
                .ok_or_else(|| {
                    "publish-crates.sh must wait for katana-language-editor-egui visibility"
                        .to_string()
                })?;

        if !(publish_core_line < wait_core_line
            && wait_core_line < publish_egui_line
            && publish_egui_line < wait_egui_line)
        {
            return Err(
                "publish-crates.sh must publish core, wait for core, publish egui, then wait for egui"
                    .to_string(),
            );
        }

        Ok(())
    }

    pub(crate) fn validate_kuc_release_set_gate_from_lines(lines: &[&str]) -> Result<(), String> {
        let gate_line =
            Self::exact_line_index(lines, "require_kuc_release_set").ok_or_else(|| {
                "publish-crates.sh must verify the KUC release set before publishing KLE"
                    .to_string()
            })?;
        let first_publish_line = lines
            .iter()
            .position(|line| line.trim_start().starts_with("publish_if_needed "))
            .ok_or_else(|| "publish-crates.sh must contain a KLE publish step".to_string())?;
        if gate_line > first_publish_line {
            return Err("KUC release-set verification must precede every KLE publish".to_string());
        }

        for required in [
            "workspace_manifest=",
            "tomllib",
            "katana-ui-core",
            "dependency_name",
            "version",
            "egui",
            "text-raster",
            "${package}@${version}",
            "cargo info",
        ] {
            if !lines.iter().any(|line| line.contains(required)) {
                return Err(format!(
                    "publish-crates.sh must derive and verify KUC package/version: {required}"
                ));
            }
        }

        if lines.iter().any(|line| {
            line.contains("publish_if_needed katana-ui-core")
                || (line.contains("cargo publish") && line.contains("katana-ui-core"))
        }) {
            return Err("publish-crates.sh must not publish KUC crates".to_string());
        }

        Ok(())
    }

    pub(crate) fn validate_completion_audit_is_read_only() -> Result<(), String> {
        Self::validate_completion_audit_read_only_from_lines(
            &RELEASE_COMPLETION.lines().collect::<Vec<_>>(),
        )?;
        Self::validate_completion_audit_read_only_from_lines(
            &RELEASE_BLOCKERS.lines().collect::<Vec<_>>(),
        )
    }

    pub(crate) fn validate_completion_audit_read_only_from_lines(
        lines: &[&str],
    ) -> Result<(), String> {
        for line in lines {
            let trimmed = line.trim_start();
            for forbidden in [
                "cargo publish",
                "gh issue create",
                "gh issue close",
                "gh issue edit",
                "gh issue reopen",
                "gh release create",
                "gh release edit",
                "git tag",
                "git push",
            ] {
                if trimmed.starts_with(forbidden) {
                    return Err(format!(
                        "verify-release-completion.sh must be read-only, but executes: {forbidden}"
                    ));
                }
            }
        }

        Ok(())
    }

    pub(crate) fn validate_release_docs_completion_boundary() -> Result<(), String> {
        Self::validate_document_contains(
            "docs/release.md",
            RELEASE_DOC,
            RELEASE_DOC_EVIDENCE,
            "release-check vs public completion boundary",
        )?;
        Self::validate_document_contains(
            "docs/v0-1-0-release-readiness.md",
            RELEASE_READINESS_DOC,
            RELEASE_READINESS_EVIDENCE,
            "current v0.1.0 incomplete release state",
        )
    }

    fn validate_document_contains(
        path: &str,
        text: &str,
        evidence: &[&str],
        purpose: &str,
    ) -> Result<(), String> {
        for needle in evidence {
            if !text.contains(needle) {
                return Err(format!("{path} is missing {purpose} evidence: {needle}"));
            }
        }
        Ok(())
    }
}
