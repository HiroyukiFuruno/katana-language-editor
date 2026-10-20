use crate::release_gate::ReleaseGateAudit;

const CANONICAL_ARTIFACT_DIRECTORY: &str =
    "artifacts/v0-1-0/source-closure-input/4f6a6287c650a38633c7baeb544a92e739c68567/artifacts";

impl ReleaseGateAudit {
    pub(crate) fn validate_source_closure_artifact_handoff_contract(
        lines: &[&str],
    ) -> Result<(), String> {
        Self::validate_source_closure_kuc_checkout_contract(lines)?;
        Self::validate_source_closure_kle_checkout_commit_contract(lines)?;
        Self::validate_source_closure_canonical_git_checkout(lines)?;
        Self::validate_source_closure_fixed_dependency_fetch(lines)?;
        Self::validate_source_closure_kle_release_parity(lines)?;
        Self::validate_source_anchor_catalog_handoff(lines)?;
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

    fn validate_source_anchor_catalog_handoff(lines: &[&str]) -> Result<(), String> {
        let assemble = Self::job_section(lines, "assemble-validate-materialize")?;
        Self::step_section(assemble, "Materialize source closure")?;
        let emit = Self::step_section(assemble, "Emit source anchor catalog")?;
        let validate = Self::step_section(assemble, "Validate source anchor catalog")?;
        let upload = Self::step_section(assemble, "Upload source anchor catalog")?;
        Self::step_section(assemble, "Emit generic interaction annotation catalog")?;
        let materialize_index = step_index(assemble, "Materialize source closure")?;
        let emit_index = step_index(assemble, "Emit source anchor catalog")?;
        let validate_index = step_index(assemble, "Validate source anchor catalog")?;
        let upload_index = step_index(assemble, "Upload source anchor catalog")?;
        let annotation_index = step_index(assemble, "Emit generic interaction annotation catalog")?;
        if !(materialize_index < emit_index
            && emit_index < validate_index
            && validate_index < upload_index
            && upload_index < annotation_index)
        {
            return Err("source anchor catalog steps must be ordered materialize < emit < validate < upload < generic annotation".into());
        }

        let canonical =
            format!("--source-closure {CANONICAL_ARTIFACT_DIRECTORY}/source-closure.json");
        let output = "--output target/source-closure/${{ env.SOURCE_CLOSURE_RUN_ID }}/source-anchor-catalog.json";
        let catalog =
            "target/source-closure/${{ env.SOURCE_CLOSURE_RUN_ID }}/source-anchor-catalog.json";
        validate_exact_step(
            emit,
            &[
                "- name: Emit source anchor catalog",
                "working-directory: katana-language-editor",
                "run: >-",
                "cargo run --locked -p katana-parity-check -- source-closure emit-source-anchors",
                &canonical,
                "--katana-repo ../katana",
                output,
            ],
            "emit",
        )?;
        let canonical = format!(
            "--source-closure {}/source-closure.json",
            CANONICAL_ARTIFACT_DIRECTORY
        );
        let catalog_argument = format!("--catalog {catalog}");
        validate_exact_step(
            validate,
            &[
                "- name: Validate source anchor catalog",
                "working-directory: katana-language-editor",
                "run: >-",
                "cargo run --locked -p katana-parity-check -- source-closure validate-source-anchors",
                &canonical,
                &catalog_argument,
            ],
            "validate",
        )?;
        validate_exact_step(
            upload,
            &[
                "- name: Upload source anchor catalog",
                "uses: actions/upload-artifact@v4",
                "with:",
                "name: source-closure-source-anchors-${{ github.run_id }}",
                "path: katana-language-editor/target/source-closure/${{ env.SOURCE_CLOSURE_RUN_ID }}/source-anchor-catalog.json",
                "if-no-files-found: error",
                "retention-days: 14",
                "overwrite: true",
            ],
            "upload",
        )?;
        Ok(())
    }

    fn validate_source_closure_kle_checkout_commit_contract(lines: &[&str]) -> Result<(), String> {
        for job_name in [
            "capture-profile",
            "assemble-validate-materialize",
            "native-host-e2e",
        ] {
            let job = Self::job_section(lines, job_name)?;
            let checkout = Self::step_section(job, "Checkout KLE")?;
            Self::exact_trimmed_line(checkout, "ref: ${{ github.sha }}").map_err(|_| {
                "source-closure KLE checkouts must pin the immutable triggering commit with github.sha".to_string()
            })?;
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

fn step_index(lines: &[&str], step_name: &str) -> Result<usize, String> {
    let expected = format!("- name: {step_name}");
    let matches = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.trim() == expected)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [index] => Ok(*index),
        [] => Err(format!("source-closure `{step_name}` step is missing")),
        _ => Err(format!("source-closure `{step_name}` step is duplicated")),
    }
}

fn validate_exact_step(step: &[&str], expected: &[&str], name: &str) -> Result<(), String> {
    let actual = step
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect::<Vec<_>>();
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "source anchor {name} step must match its exact fail-closed shape"
        ))
    }
}
