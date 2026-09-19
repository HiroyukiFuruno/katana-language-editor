use crate::release_gate::ReleaseGateAudit;
use crate::release_gate_sources::SOURCE_CLOSURE_WORKFLOW;

const ASSEMBLED_ARTIFACT: &str =
    "source-closure-assembled-${{ github.run_id }}-${{ github.run_attempt }}";

impl ReleaseGateAudit {
    pub(crate) fn validate_source_closure_native_host_contract() -> Result<(), String> {
        Self::validate_source_closure_native_host_contract_from_lines(
            &SOURCE_CLOSURE_WORKFLOW.lines().collect::<Vec<_>>(),
        )
    }

    pub(crate) fn validate_source_closure_native_host_contract_from_lines(
        lines: &[&str],
    ) -> Result<(), String> {
        let assemble = Self::job_section(lines, "assemble-validate-materialize")?;
        let native = Self::job_section(lines, "native-host-e2e")?;
        Self::exact_trimmed_line(native, "needs: assemble-validate-materialize")?;
        Self::exact_trimmed_line(
            native,
            "if: github.event_name == 'workflow_dispatch' && github.ref_protected",
        )?;
        Self::exact_trimmed_line(native, "runs-on: [self-hosted, macOS, katana-native-e2e]")?;

        validate_native_job_restrictions(native)?;
        Self::validate_source_closure_artifact_handoff_contract(lines)?;
        Self::validate_fixed_source_closure_katana_checkout(native)?;
        Self::validate_source_closure_assemble_input_contract(lines)?;
        Self::validate_source_closure_provenance_capture_contract(lines)?;
        validate_physical_test_step(native)?;
        validate_full_editor_parity_gate_step(native)?;
        validate_materialize_step(assemble)?;
        Self::validate_source_closure_input_recipes()?;
        Self::validate_source_closure_materialize_recipe()?;

        let uploaded_artifact = Self::artifact_name_in_step(
            assemble,
            "uses: actions/upload-artifact@v4",
            "assemble-validate-materialize",
        )?;
        let downloaded_artifact = Self::artifact_name_in_step(
            native,
            "uses: actions/download-artifact@v4",
            "native-host-e2e",
        )?;
        if uploaded_artifact != ASSEMBLED_ARTIFACT || downloaded_artifact != ASSEMBLED_ARTIFACT {
            return Err(
                "source-closure native host E2E must consume the same run assembled artifact"
                    .to_string(),
            );
        }
        Ok(())
    }
}

fn validate_materialize_step(lines: &[&str]) -> Result<(), String> {
    let step = ReleaseGateAudit::step_section(lines, "Materialize source closure")?;
    let command = step
        .iter()
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join(" ");
    for expected in [
        "source-closure materialize-closure",
        "--input target/source-closure/${{ env.SOURCE_CLOSURE_RUN_ID }}/assembled/source-closure-input.json",
        "--katana-repo ../katana",
        "--canonical-root artifacts",
    ] {
        if !command.contains(expected) {
            return Err(format!(
                "source-closure materialize step is missing `{expected}`"
            ));
        }
    }
    if command.contains("--artifact-dir") {
        return Err(
            "source-closure materialize step uses unsupported `--artifact-dir`".to_string(),
        );
    }
    Ok(())
}

fn validate_physical_test_step(lines: &[&str]) -> Result<(), String> {
    let step = ReleaseGateAudit::step_section(lines, "Run physical native host E2E")?;
    let command = step
        .iter()
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join(" ");
    for expected in [
        "cargo test",
        "--manifest-path tools/katana-host-e2e/Cargo.toml",
        "--test physical_open_workspace",
    ] {
        if !command.contains(expected) {
            return Err(format!(
                "physical native host E2E command is missing `{expected}`"
            ));
        }
    }
    Ok(())
}

fn validate_full_editor_parity_gate_step(lines: &[&str]) -> Result<(), String> {
    let step = ReleaseGateAudit::step_section(lines, "Run full KatanA editor parity gate")?;
    let command = step
        .iter()
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join(" ");
    let expected = "KATANA_REPO=../katana just full-parity-check";
    if !command.contains(expected) {
        return Err(format!(
            "full KatanA editor parity gate is missing `{expected}`"
        ));
    }
    Ok(())
}

fn validate_native_job_restrictions(lines: &[&str]) -> Result<(), String> {
    const FORBIDDEN: &[&str] = &[
        "KATANA_OPEN_WORKSPACE_",
        "KATANA_WORKSPACE_PATH",
        "KATANA_TARGET_PATH",
        "KATANA_NATIVE_TARGET",
        "KATANA_PID",
        "KATANA_COORDINATE",
        "continue-on-error:",
        "if: always()",
        "if-no-files-found: ignore",
        "if-no-files-found: warn",
        "|| true",
        "|| echo",
        "|| exit",
        "; exit 0",
        "if: cancelled()",
        "if: false",
        "exit 0",
    ];
    if let Some((_, forbidden)) = lines
        .iter()
        .flat_map(|line| FORBIDDEN.iter().map(move |forbidden| (*line, *forbidden)))
        .find(|(line, forbidden)| line.contains(forbidden))
    {
        return Err(format!(
            "source-closure native host E2E contains forbidden control or fallback structure `{forbidden}`"
        ));
    }
    if lines.iter().any(|line| {
        let trimmed = line.trim();
        trimmed == "env:" || trimmed.starts_with("env:")
    }) {
        return Err(
            "source-closure native host E2E must not receive environment-variable inputs"
                .to_string(),
        );
    }
    if lines.iter().any(|line| {
        let trimmed = line.trim();
        (trimmed.starts_with("push:")
            || trimmed.starts_with("pull_request:")
            || trimmed.contains("github.event_name == 'push'")
            || trimmed.contains("github.event_name == 'pull_request'"))
            && !trimmed.contains("workflow_dispatch")
    }) {
        return Err(
            "source-closure native host E2E must be protected workflow_dispatch-only".to_string(),
        );
    }
    let if_lines = lines
        .iter()
        .filter(|line| line.trim_start().starts_with("if:"))
        .map(|line| line.trim())
        .collect::<Vec<_>>();
    if if_lines != ["if: github.event_name == 'workflow_dispatch' && github.ref_protected"] {
        return Err(
            "source-closure native host E2E must have exactly one protected workflow_dispatch condition"
                .to_string(),
        );
    }
    Ok(())
}
