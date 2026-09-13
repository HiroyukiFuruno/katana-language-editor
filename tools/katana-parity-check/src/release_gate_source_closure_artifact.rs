use crate::release_gate::ReleaseGateAudit;

const CANONICAL_ARTIFACT_DIRECTORY: &str =
    "artifacts/v0-1-0/source-closure-input/4f6a6287c650a38633c7baeb544a92e739c68567/artifacts";

impl ReleaseGateAudit {
    pub(crate) fn validate_source_closure_artifact_handoff_contract(
        lines: &[&str],
    ) -> Result<(), String> {
        Self::validate_source_closure_kuc_checkout_contract(lines)?;
        let native = Self::job_section(lines, "native-host-e2e")?;
        let step = Self::step_section(native, "Run full KatanA editor parity gate")?;
        let command = step
            .iter()
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join(" ");
        let expected =
            format!("KATANA_PARITY_SOURCE_CLOSURE_ARTIFACT_DIR={CANONICAL_ARTIFACT_DIRECTORY}");
        if !command.contains(&expected) {
            return Err(format!(
                "source-closure native parity gate is missing canonical artifact directory `{CANONICAL_ARTIFACT_DIRECTORY}`"
            ));
        }
        Ok(())
    }
}
