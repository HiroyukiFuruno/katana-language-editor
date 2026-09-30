use crate::release_gate::ReleaseGateAudit;
use crate::release_gate_sources::RELEASE_WORKFLOW;

impl ReleaseGateAudit {
    pub(crate) fn validate_release_workflow_ordering() -> Result<(), String> {
        let lines: Vec<&str> = RELEASE_WORKFLOW.lines().collect();
        Self::validate_release_workflow_manual_guard_from_lines(&lines)?;
        Self::validate_release_source_closure_handoff_from_lines(&lines)?;
        let names = [
            "Reject partial manual release",
            "Release check",
            "Create release tag",
            "Create GitHub Release",
            "Publish crates.io",
            "Verify public release completion",
        ];
        let indices = names
            .iter()
            .map(|name| {
                Self::line_index(&lines, &format!("      - name: {name}"))
                    .ok_or_else(|| format!("Release workflow is missing {name}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if !indices.windows(2).all(|pair| pair[0] < pair[1]) {
            return Err(
                "Release workflow must run release gates and public completion audits in order"
                    .to_string(),
            );
        }
        Ok(())
    }

    pub(crate) fn validate_release_source_closure_handoff_from_lines(
        lines: &[&str],
    ) -> Result<(), String> {
        let source_closure = Self::job_section(lines, "source-closure")?;
        for expected in [
            "if: >",
            "github.event_name == 'workflow_dispatch' || (",
            "github.event.pull_request.merged == true &&",
            "github.event.pull_request.head.repo.full_name == github.repository &&",
            "startsWith(github.event.pull_request.head.ref, 'release/v')",
            ")",
        ] {
            if !source_closure.iter().any(|line| line.trim() == expected) {
                return Err(format!(
                    "Release source-closure must use the release predicate: missing {expected}"
                ));
            }
        }

        let release = Self::job_section(lines, "release")?;
        let checkout = Self::step_section(release, "Checkout")?;
        Self::exact_trimmed_line(checkout, "ref: ${{ github.sha }}").map_err(|_| {
            "Release checkout must pin the immutable triggering commit with github.sha".to_string()
        })?;

        let required = [
            "  source-closure:",
            "    uses: ./.github/workflows/source-closure.yml",
            "    needs: source-closure",
            "      - name: Download validated source-closure evidence",
            "          name: source-closure-assembled-${{ github.run_id }}-${{ github.run_attempt }}",
            "          path: katana-language-editor",
            "          KATANA_PARITY_SOURCE_CLOSURE_ARTIFACT_DIR: artifacts/v0-1-0/source-closure-input/4f6a6287c650a38633c7baeb544a92e739c68567/artifacts",
        ];
        for needle in required {
            if !lines.contains(&needle) {
                return Err(format!(
                    "Release workflow is missing same-run source-closure handoff evidence: {needle}"
                ));
            }
        }

        let download = Self::line_index(
            lines,
            "      - name: Download validated source-closure evidence",
        )
        .ok_or_else(|| {
            "Release workflow is missing source-closure artifact download".to_string()
        })?;
        let release_check = Self::line_index(lines, "      - name: Release check")
            .ok_or_else(|| "Release workflow is missing release check".to_string())?;
        if download >= release_check {
            return Err(
                "Release workflow must download same-run source-closure evidence before release-check"
                    .to_string(),
            );
        }
        Ok(())
    }

    pub(crate) fn validate_release_workflow_manual_guard_from_lines(
        lines: &[&str],
    ) -> Result<(), String> {
        let input_line = Self::line_index(lines, "      publish_crates:")
            .ok_or_else(|| "Release workflow is missing publish_crates input".to_string())?;
        let default_true_line = lines
            .iter()
            .enumerate()
            .skip(input_line + 1)
            .take_while(|(_, line)| {
                !line.starts_with("      - ") && !line.starts_with("      version:")
            })
            .find(|(_, line)| line.trim() == "default: true")
            .map(|(index, _)| index)
            .ok_or_else(|| "publish_crates must default to true for manual releases".to_string())?;
        if default_true_line <= input_line {
            return Err("publish_crates default must be declared after the input".to_string());
        }

        let guard_line = Self::line_index(lines, "      - name: Reject partial manual release")
            .ok_or_else(|| {
                "Release workflow is missing partial manual release rejection before mutation"
                    .to_string()
            })?;
        let guard_if_line = lines
            .get(guard_line + 1)
            .map(|line| line.trim())
            .ok_or_else(|| "Partial manual release guard is missing an if condition".to_string())?;
        if guard_if_line
            != "if: github.event_name == 'workflow_dispatch' && inputs.publish_crates != true"
        {
            return Err(
                "Partial manual release guard must reject workflow_dispatch with publish_crates != true"
                    .to_string(),
            );
        }

        let release_check_line = Self::line_index(lines, "      - name: Release check")
            .ok_or_else(|| "Release workflow is missing release check".to_string())?;
        if guard_line >= release_check_line {
            return Err(
                "Partial manual release guard must run before release-check or release mutation"
                    .to_string(),
            );
        }
        Ok(())
    }
}
