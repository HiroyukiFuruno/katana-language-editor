use crate::release_gate::ReleaseGateAudit;

const NATIVE_HOST_LAYOUT_ARTIFACT: &str = "source-closure-native-host-e2e-${{ github.run_id }}";

pub(crate) struct NativeHostStepAudit;

impl NativeHostStepAudit {
    pub(crate) fn validate_native_host_layout_handoff(
        assemble: &[&str],
        native: &[&str],
    ) -> Result<(), String> {
        let required_assemble = [
            "- name: Prepare native host E2E layout",
            "target/source-closure/native-host-e2e",
            "artifacts/source-derived-native-target.json",
            "artifacts/context-menu-target-manifest.json",
            "profiles/macos-latest/probe.json",
            "name: source-closure-native-host-e2e-${{ github.run_id }}",
        ];
        for needle in required_assemble {
            if !assemble
                .iter()
                .any(|line| line.trim() == needle || line.contains(needle))
            {
                return Err(format!(
                    "source-closure assemble job is missing native host layout evidence `{needle}`"
                ));
            }
        }
        let required_native = [
            "- name: Download native host E2E layout",
            "name: source-closure-native-host-e2e-${{ github.run_id }}",
            "path: katana-language-editor/target/source-closure",
        ];
        for needle in required_native {
            if !native.iter().any(|line| line.trim() == needle) {
                return Err(format!(
                    "source-closure native job is missing native host layout evidence `{needle}`"
                ));
            }
        }
        if !native
            .iter()
            .any(|line| line.contains(NATIVE_HOST_LAYOUT_ARTIFACT))
        {
            return Err(
                "source-closure native job must download the same-run native host layout artifact"
                    .to_string(),
            );
        }
        Ok(())
    }

    pub(crate) fn validate_materialize_step(lines: &[&str]) -> Result<(), String> {
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

    pub(crate) fn validate_physical_test_step(lines: &[&str]) -> Result<(), String> {
        let step = ReleaseGateAudit::step_section(lines, "Run physical native host E2E")?;
        let command = step
            .iter()
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join(" ");
        for expected in [
            "KATANA_REPO=../katana",
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

    pub(crate) fn validate_full_editor_parity_gate_step(lines: &[&str]) -> Result<(), String> {
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
}
