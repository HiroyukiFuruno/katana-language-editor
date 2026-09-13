use crate::release_gate::ReleaseGateAudit;
use crate::release_gate_sources::TEST_AND_BUILD_WORKFLOW;

#[test]
fn test_and_build_workflow_enforces_strict_unit_and_coverage_routes() -> Result<(), String> {
    ReleaseGateAudit::validate_test_and_build_workflow()
}

#[test]
fn test_and_build_workflow_rejects_ignored_failures() {
    for (anchor, replacement) in [
        (
            "        run: just coverage",
            "        continue-on-error: true\n        run: just coverage",
        ),
        ("  test:", "  test:\n    continue-on-error: true"),
    ] {
        assert!(TEST_AND_BUILD_WORKFLOW.contains(anchor));
        let workflow = TEST_AND_BUILD_WORKFLOW.replacen(anchor, replacement, 1);
        let lines = workflow.lines().collect::<Vec<_>>();
        let result = ReleaseGateAudit::validate_test_and_build_workflow_from_lines(&lines);
        assert!(matches!(result, Err(error) if error.contains("must not ignore")));
    }
}

#[test]
fn test_and_build_workflow_rejects_missing_commands() {
    for command in ["run: just unit-test", "run: just coverage"] {
        assert!(TEST_AND_BUILD_WORKFLOW.contains(command));
        let workflow = TEST_AND_BUILD_WORKFLOW.replacen(command, "run: just help", 1);
        let lines = workflow.lines().collect::<Vec<_>>();
        let result = ReleaseGateAudit::validate_test_and_build_workflow_from_lines(&lines);
        assert!(matches!(result, Err(error) if error.contains("must run")));
    }
}
