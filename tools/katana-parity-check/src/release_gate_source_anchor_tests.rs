use crate::release_gate::ReleaseGateAudit;

fn fixture() -> String {
    crate::release_gate_sources::SOURCE_CLOSURE_WORKFLOW.to_string()
}

fn assert_rejected(workflow: String) -> Result<(), String> {
    let lines = workflow.lines().collect::<Vec<_>>();
    ReleaseGateAudit::validate_source_closure_native_host_contract_from_lines(&lines)
        .err()
        .map(|_| ())
        .ok_or_else(|| "expected source-anchor workflow mutation to be rejected".into())
}

fn assert_mutation_rejected(original: &str, mutated: String) -> Result<(), String> {
    if mutated == original {
        return Err("workflow mutation did not change the fixture".into());
    }
    assert_rejected(mutated)
}

#[test]
fn source_anchor_workflow_contract_accepts_current_workflow() -> Result<(), String> {
    let lines = fixture().lines().map(str::to_owned).collect::<Vec<_>>();
    let refs = lines.iter().map(String::as_str).collect::<Vec<_>>();
    ReleaseGateAudit::validate_source_closure_native_host_contract_from_lines(&refs)
}

#[test]
fn source_anchor_workflow_rejects_missing_and_reordered_steps() -> Result<(), String> {
    let original = fixture();
    assert_mutation_rejected(
        &original,
        original.replace(
            "      - name: Emit source anchor catalog",
            "      - name: Skip source anchor catalog",
        ),
    )?;
    let emit_start = original
        .find("      - name: Emit source anchor catalog")
        .ok_or("emit marker missing")?;
    let annotation_start = original
        .find("      - name: Emit generic interaction annotation catalog")
        .ok_or("annotation marker missing")?;
    let emit = &original[emit_start..annotation_start];
    let rest = &original[annotation_start..];
    let validation_start = emit
        .find("      - name: Validate source anchor catalog")
        .ok_or("validate marker missing")?;
    let upload_start = emit
        .find("      - name: Upload source anchor catalog")
        .ok_or("upload marker missing")?;
    let chunks = [
        &emit[..validation_start],
        &emit[validation_start..upload_start],
        &emit[upload_start..],
    ];
    assert_mutation_rejected(
        &original,
        format!(
            "{}{}{}{}{}",
            &original[..emit_start],
            chunks[1],
            chunks[0],
            chunks[2],
            rest
        ),
    )
}

#[test]
fn source_anchor_workflow_rejects_authority_output_and_skip_mutations() -> Result<(), String> {
    let original = fixture();
    for mutated in [
        original.replace("4f6a6287c650a38633c7baeb544a92e739c68567/artifacts/source-closure.json", "other/source-closure.json"),
        original.replace("source-anchor-catalog.json", "wrong-catalog.json"),
        original.replace("          cargo run --locked -p katana-parity-check -- source-closure validate-source-anchors", "          echo skip && cargo run --locked -p katana-parity-check -- source-closure validate-source-anchors"),
        original.replace("      - name: Validate source anchor catalog\n        working-directory:", "      - name: Validate source anchor catalog\n        if: always()\n        working-directory:"),
    ] {
        assert_mutation_rejected(&original, mutated)?;
    }
    Ok(())
}

#[test]
fn source_anchor_workflow_rejects_upload_target_mismatch() -> Result<(), String> {
    let original = fixture();
    assert_mutation_rejected(&original, original.replace(
        "path: katana-language-editor/target/source-closure/${{ env.SOURCE_CLOSURE_RUN_ID }}/source-anchor-catalog.json",
        "path: katana-language-editor/target/source-closure/other/source-anchor-catalog.json",
    ))
}

#[test]
fn source_anchor_workflow_rejects_extra_shell_and_duplicate_fields() -> Result<(), String> {
    let original = fixture();
    for mutated in [
        original.replace(
            "        run: >-\n          cargo run --locked -p katana-parity-check -- source-closure emit-source-anchors",
            "        env:\n          EXTRA: true\n        run: >-\n          cargo run --locked -p katana-parity-check -- source-closure emit-source-anchors",
        ),
        original.replace(
            "          cargo run --locked -p katana-parity-check -- source-closure validate-source-anchors",
            "          echo skip\n          cargo run --locked -p katana-parity-check -- source-closure validate-source-anchors",
        ),
        original.replace(
            "          path: katana-language-editor/target/source-closure/${{ env.SOURCE_CLOSURE_RUN_ID }}/source-anchor-catalog.json\n          if-no-files-found:",
            "          path: katana-language-editor/target/source-closure/${{ env.SOURCE_CLOSURE_RUN_ID }}/source-anchor-catalog.json\n          path: other/catalog.json\n          if-no-files-found:",
        ),
        original.replace(
            "      - name: Validate source anchor catalog\n",
            "      - name: Validate source anchor catalog\n      - name: Validate source anchor catalog\n",
        ),
    ] {
        assert_mutation_rejected(&original, mutated)?;
    }
    Ok(())
}
