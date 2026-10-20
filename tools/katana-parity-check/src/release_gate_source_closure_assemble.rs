use crate::release_gate::ReleaseGateAudit;

impl ReleaseGateAudit {
    pub(crate) fn validate_source_closure_assemble_input_contract(
        lines: &[&str],
    ) -> Result<(), String> {
        let assemble = Self::job_section(lines, "assemble-validate-materialize")?;
        let step = Self::step_section(assemble, "Assemble canonical input")?;
        let command = step
            .iter()
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join(" ");
        for expected in [
            "source-closure assemble-input",
            "--seed-manifest docs/v0-1-0-source-closure-roots.json",
            "--source-universe docs/v0-1-0-katana-editor-source-universe.md",
            "--katana-repo ../katana",
        ] {
            if !command.contains(expected) {
                return Err(format!(
                    "source-closure assemble step is missing `{expected}`"
                ));
            }
        }
        if command.contains("--seed-path") {
            return Err("source-closure assemble step bypasses the root manifest".into());
        }
        Ok(())
    }
}
