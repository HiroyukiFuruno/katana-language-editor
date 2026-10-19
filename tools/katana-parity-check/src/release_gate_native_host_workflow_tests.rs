use crate::release_gate::ReleaseGateAudit;
use crate::release_gate_sources::SOURCE_CLOSURE_WORKFLOW;

#[test]
fn native_host_contract_rejects_missing_same_run_layout_artifact() {
    let workflow = SOURCE_CLOSURE_WORKFLOW.replace(
        "source-closure-native-host-e2e-${{ github.run_id }}",
        "source-closure-native-host-e2e-stale-run",
    );
    let lines = workflow.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_source_closure_native_host_contract_from_lines(&lines);
    assert!(matches!(result, Err(error) if error.contains("native host layout")));
}

#[test]
fn native_host_contract_rejects_attempt_qualified_layout_artifact() {
    let workflow = SOURCE_CLOSURE_WORKFLOW.replace(
        "source-closure-native-host-e2e-${{ github.run_id }}",
        "source-closure-native-host-e2e-${{ github.run_id }}-${{ github.run_attempt }}",
    );
    let lines = workflow.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_source_closure_native_host_contract_from_lines(&lines);
    assert!(matches!(result, Err(error) if error.contains("native host layout")));
}

#[test]
fn native_host_contract_rejects_missing_mapping_input() {
    let workflow = SOURCE_CLOSURE_WORKFLOW.replace(
        "      - name: Upload source mapping input",
        "      - name: Skip source mapping input",
    );
    let lines = workflow.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_source_closure_native_host_contract_from_lines(&lines);
    assert!(matches!(result, Err(error) if error.contains("Upload source mapping input")));
}

#[test]
fn source_closure_reruns_keep_capture_identity_and_replace_artifacts() -> Result<(), String> {
    assert!(SOURCE_CLOSURE_WORKFLOW.contains("SOURCE_CLOSURE_RUN_ID: ${{ github.run_id }}"));
    assert!(
        !SOURCE_CLOSURE_WORKFLOW
            .contains("SOURCE_CLOSURE_RUN_ID: ${{ github.run_id }}-${{ github.run_attempt }}")
    );
    for artifact in [
        "source-closure-generic-interaction-annotations-${{ github.run_id }}",
        "source-closure-mapping-input-${{ github.run_id }}",
        "source-closure-assembled-${{ github.run_id }}",
        "source-closure-native-host-e2e-${{ github.run_id }}",
    ] {
        assert_upload_is_replaced(artifact)?;
    }
    Ok(())
}

fn assert_upload_is_replaced(artifact_name: &str) -> Result<(), String> {
    let start = SOURCE_CLOSURE_WORKFLOW
        .find(artifact_name)
        .ok_or_else(|| format!("missing source-closure upload artifact {artifact_name}"))?;
    let step = &SOURCE_CLOSURE_WORKFLOW[start..];
    let step = step
        .split("      - name:")
        .next()
        .ok_or_else(|| format!("missing source-closure upload step for {artifact_name}"))?;
    if step.contains("overwrite: true") {
        Ok(())
    } else {
        Err(format!(
            "source-closure upload artifact must overwrite on rerun: {artifact_name}"
        ))
    }
}
