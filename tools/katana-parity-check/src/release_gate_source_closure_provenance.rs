use crate::release_gate::ReleaseGateAudit;

impl ReleaseGateAudit {
    pub(crate) fn validate_source_closure_provenance_capture_contract(
        lines: &[&str],
    ) -> Result<(), String> {
        let assemble = Self::job_section(lines, "assemble-validate-materialize")?;
        let step = Self::step_section(assemble, "Capture same-run read-only provenance")?;
        let command = step
            .iter()
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join(" ");
        for expected in [
            "source-closure capture-provenance",
            "--source-universe docs/v0-1-0-katana-editor-source-universe.md",
            "--requirement-source-aliases docs/v0-1-0-editor-requirement-source-aliases.json",
        ] {
            if !command.contains(expected) {
                return Err(format!(
                    "source-closure provenance capture step is missing `{expected}`"
                ));
            }
        }
        Ok(())
    }
}
