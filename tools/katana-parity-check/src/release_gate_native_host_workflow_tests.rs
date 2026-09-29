use crate::release_gate::ReleaseGateAudit;
use crate::release_gate_sources::SOURCE_CLOSURE_WORKFLOW;

#[test]
fn native_host_contract_rejects_missing_same_run_layout_artifact() {
    let workflow = SOURCE_CLOSURE_WORKFLOW.replace(
        "source-closure-native-host-e2e-${{ github.run_id }}-${{ github.run_attempt }}",
        "source-closure-native-host-e2e-stale-run",
    );
    let lines = workflow.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_source_closure_native_host_contract_from_lines(&lines);
    assert!(matches!(result, Err(error) if error.contains("native host layout")));
}
