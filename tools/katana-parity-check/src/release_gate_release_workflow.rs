use crate::release_gate::ReleaseGateAudit;
use crate::release_gate_sources::RELEASE_WORKFLOW;

impl ReleaseGateAudit {
    pub(crate) fn validate_release_workflow_ordering() -> Result<(), String> {
        let lines: Vec<&str> = RELEASE_WORKFLOW.lines().collect();
        Self::validate_release_workflow_manual_guard_from_lines(&lines)?;
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
