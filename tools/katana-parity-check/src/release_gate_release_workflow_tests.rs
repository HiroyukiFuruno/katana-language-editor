use crate::release_gate::ReleaseGateAudit;
use crate::release_gate_sources::RELEASE_WORKFLOW;

fn assert_error_contains(result: Result<(), String>, expected: &str) -> Result<(), String> {
    match result {
        Ok(()) => Err(format!("expected error containing {expected}, got Ok")),
        Err(error) if error.contains(expected) => Ok(()),
        Err(error) => Err(format!("expected error containing {expected}, got {error}")),
    }
}

#[test]
fn release_workflow_requires_same_run_source_closure_handoff() -> Result<(), String> {
    let workflow = RELEASE_WORKFLOW.replace("    needs: source-closure\n", "");
    let lines = workflow.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_release_source_closure_handoff_from_lines(&lines);
    assert_error_contains(result, "needs: source-closure")
}

#[test]
fn release_workflow_requires_release_predicate_for_source_closure() -> Result<(), String> {
    let workflow = RELEASE_WORKFLOW.replace(
        "  source-closure:\n    if: >\n      github.event_name == 'workflow_dispatch' || (\n        github.event.pull_request.merged == true &&\n        github.event.pull_request.head.repo.full_name == github.repository &&\n        startsWith(github.event.pull_request.head.ref, 'release/v')\n      )\n    uses: ./.github/workflows/source-closure.yml\n",
        "  source-closure:\n    uses: ./.github/workflows/source-closure.yml\n",
    );
    let lines = workflow.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_release_source_closure_handoff_from_lines(&lines);
    assert_error_contains(
        result,
        "Release source-closure must use the release predicate",
    )
}

#[test]
fn release_workflow_requires_immutable_checkout_for_source_closure_evidence() -> Result<(), String>
{
    let workflow = RELEASE_WORKFLOW.replace("ref: ${{ github.sha }}", "ref: master");
    let lines = workflow.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_release_source_closure_handoff_from_lines(&lines);
    assert_error_contains(result, "immutable triggering commit")
}

#[test]
fn release_workflow_rejects_source_closure_artifact_from_another_run() -> Result<(), String> {
    let workflow = RELEASE_WORKFLOW.replace(
        "source-closure-assembled-${{ github.run_id }}-${{ github.run_attempt }}",
        "source-closure-assembled-stale-run",
    );
    let lines = workflow.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_release_source_closure_handoff_from_lines(&lines);
    assert_error_contains(result, "source-closure-assembled-${{ github.run_id }}")
}

#[test]
fn release_workflow_rejects_release_check_before_source_closure_download() -> Result<(), String> {
    let download = "      - name: Download validated source-closure evidence\n        uses: actions/download-artifact@v4\n        with:\n          name: source-closure-assembled-${{ github.run_id }}-${{ github.run_attempt }}\n          path: katana-language-editor\n\n";
    let without_download = RELEASE_WORKFLOW.replace(download, "");
    let reordered = without_download.replace(
        "      - name: Create release tag\n",
        &format!("{download}      - name: Create release tag\n"),
    );
    let lines = reordered.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_release_source_closure_handoff_from_lines(&lines);
    assert_error_contains(result, "before release-check")
}
