use crate::release_gate::ReleaseGateAudit;

const CANONICAL_ARTIFACT_DIRECTORY: &str =
    "artifacts/v0-1-0/source-closure-input/4f6a6287c650a38633c7baeb544a92e739c68567/artifacts";

impl ReleaseGateAudit {
    pub(crate) fn validate_source_closure_artifact_handoff_contract(
        lines: &[&str],
    ) -> Result<(), String> {
        Self::validate_source_closure_kuc_checkout_contract(lines)?;
        Self::validate_source_closure_canonical_git_checkout(lines)?;
        Self::validate_source_closure_fixed_dependency_fetch(lines)?;
        Self::validate_source_closure_kle_release_parity(lines)?;
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

    fn validate_source_closure_kle_release_parity(lines: &[&str]) -> Result<(), String> {
        let assemble = Self::job_section(lines, "assemble-validate-materialize")?;
        let step = Self::step_section(assemble, "Validate KLE release parity")?;
        let artifact =
            format!("KATANA_PARITY_SOURCE_CLOSURE_ARTIFACT_DIR: {CANONICAL_ARTIFACT_DIRECTORY}");
        if !assemble.iter().any(|line| line.trim() == artifact)
            || !step.iter().any(|line| {
                line.trim()
                    == "run: cargo run --locked -p katana-parity-check -- --mode kle-release"
            })
        {
            return Err("source-closure assemble job must validate KLE release parity using the canonical artifact directory".into());
        }
        Ok(())
    }

    fn validate_source_closure_canonical_git_checkout(lines: &[&str]) -> Result<(), String> {
        let capture = Self::job_section(lines, "capture-profile")?;
        let step = Self::step_section(capture, "Disable Git line-ending conversion")?;
        let expected = "run: git config --global core.autocrlf false";
        if !step.iter().any(|line| line.trim() == expected) {
            return Err(format!(
                "source-closure capture-profile must run `{expected}` before checkout"
            ));
        }
        Ok(())
    }

    fn validate_source_closure_fixed_dependency_fetch(lines: &[&str]) -> Result<(), String> {
        let capture = Self::job_section(lines, "capture-profile")?;
        let step = Self::step_section(capture, "Fetch fixed KatanA dependencies")?;
        let expected = "run: cargo fetch --locked --manifest-path ../katana/Cargo.toml";
        if !step.iter().any(|line| line.trim() == expected) {
            return Err(format!(
                "source-closure capture-profile must run `{expected}` before offline metadata capture"
            ));
        }
        Ok(())
    }
}
