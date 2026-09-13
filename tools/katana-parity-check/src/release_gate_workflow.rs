use crate::release_gate::ReleaseGateAudit;
use crate::release_gate_sources::TEST_AND_BUILD_WORKFLOW;

#[cfg(test)]
#[path = "release_gate_test_workflow_tests.rs"]
mod tests;

impl ReleaseGateAudit {
    pub(crate) fn validate_test_and_build_workflow() -> Result<(), String> {
        Self::validate_test_and_build_workflow_from_lines(
            &TEST_AND_BUILD_WORKFLOW.lines().collect::<Vec<_>>(),
        )
    }

    pub(crate) fn validate_test_and_build_workflow_from_lines(
        lines: &[&str],
    ) -> Result<(), String> {
        if lines
            .iter()
            .any(|line| line.trim_start().starts_with("continue-on-error:"))
        {
            return Err("test-and-build workflow must not ignore job or step failures".to_string());
        }

        let test_job = Self::job_section(lines, "test")?;
        let tests = Self::step_section(test_job, "Run tests")?;
        if !tests
            .iter()
            .any(|line| line.trim() == "run: just unit-test")
        {
            return Err("test-and-build Run tests step must run `just unit-test`".to_string());
        }

        let coverage = Self::step_section(test_job, "Run coverage")?;
        Self::validate_mandatory_coverage_step(coverage)
    }

    fn validate_mandatory_coverage_step(coverage: &[&str]) -> Result<(), String> {
        if !coverage
            .iter()
            .any(|line| line.trim() == "if: matrix.os == 'ubuntu-latest'")
        {
            return Err(
                "test-and-build Run coverage step must be mandatory on ubuntu-latest".to_string(),
            );
        }
        if !coverage
            .iter()
            .any(|line| line.trim() == "run: just coverage")
        {
            return Err("test-and-build Run coverage step must run `just coverage`".to_string());
        }
        Ok(())
    }
}

impl ReleaseGateAudit {
    pub(crate) fn job_section<'a>(
        lines: &'a [&'a str],
        job: &str,
    ) -> Result<&'a [&'a str], String> {
        let header = format!("  {job}:");
        let starts = lines
            .iter()
            .enumerate()
            .filter_map(|(index, line)| (*line == header).then_some(index))
            .collect::<Vec<_>>();
        if starts.is_empty() {
            return Err(format!("source-closure workflow is missing {job} job"));
        }
        if starts.len() != 1 {
            return Err(format!(
                "source-closure workflow must define exactly one {job} job"
            ));
        }
        let start = starts[0];
        let end = lines
            .iter()
            .enumerate()
            .skip(start + 1)
            .find(|(_, line)| line.starts_with("  ") && !line.starts_with("    "))
            .map(|(index, _)| index)
            .unwrap_or(lines.len());
        if end <= start + 1 {
            return Err(format!(
                "source-closure workflow {job} job must not be empty"
            ));
        }
        Ok(&lines[start + 1..end])
    }

    pub(crate) fn exact_trimmed_line<'a>(
        lines: &[&'a str],
        expected: &str,
    ) -> Result<&'a str, String> {
        lines
            .iter()
            .find(|line| line.trim() == expected)
            .copied()
            .ok_or_else(|| format!("source-closure native host E2E is missing `{expected}`"))
    }

    pub(crate) fn step_section<'a>(
        lines: &'a [&'a str],
        step_name: &str,
    ) -> Result<&'a [&'a str], String> {
        let header = format!("      - name: {step_name}");
        let start = lines
            .iter()
            .position(|line| *line == header)
            .ok_or_else(|| format!("source-closure workflow is missing `{step_name}` step"))?;
        let end = lines
            .iter()
            .enumerate()
            .skip(start + 1)
            .find(|(_, line)| line.starts_with("      - name:"))
            .map(|(index, _)| index)
            .unwrap_or(lines.len());
        Ok(&lines[start..end])
    }

    pub(crate) fn artifact_name_in_step<'a>(
        lines: &'a [&'a str],
        action: &str,
        job: &str,
    ) -> Result<&'a str, String> {
        let action_index = lines
            .iter()
            .position(|line| line.trim() == action)
            .ok_or_else(|| format!("{job} job is missing `{action}`"))?;
        let end = lines
            .iter()
            .enumerate()
            .skip(action_index + 1)
            .find(|(_, line)| line.starts_with("      - name:"))
            .map(|(index, _)| index)
            .unwrap_or(lines.len());
        let step = &lines[action_index..end];
        step.iter()
            .find(|line| line.trim().starts_with("name:"))
            .map(|line| line.trim().trim_start_matches("name: "))
            .ok_or_else(|| format!("{job} job artifact step is missing an artifact name"))
    }
}
