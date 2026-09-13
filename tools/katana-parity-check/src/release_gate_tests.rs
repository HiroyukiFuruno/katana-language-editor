use crate::release_gate::ReleaseGateAudit;

fn assert_error_contains(result: Result<(), String>, expected: &str) -> Result<(), String> {
    match result {
        Ok(()) => Err(format!("expected error containing {expected}, got Ok")),
        Err(error) if error.contains(expected) => Ok(()),
        Err(error) => Err(format!("expected error containing {expected}, got {error}")),
    }
}

#[test]
fn justfile_keeps_release_check_independent_of_downstream_adapters() -> Result<(), String> {
    ReleaseGateAudit::validate()
}

#[test]
fn justfile_keeps_kle_motion_and_ast_boundary_gates() -> Result<(), String> {
    ReleaseGateAudit::validate_kle_release_gate()
}

#[test]
fn kle_release_gate_rejects_the_stale_motion_test() -> Result<(), String> {
    let lines = crate::release_gate_sources::JUSTFILE.replace(
        "storybook_motion_artifact_uses_the_exact_kuc_plan_without_idle_frames -- --test-threads=1",
        "motion_artifact -- --test-threads=1",
    );
    let lines = lines.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_kle_release_gate_from_lines(&lines);
    assert_error_contains(
        result,
        "storybook_motion_artifact_uses_the_exact_kuc_plan_without_idle_frames",
    )
}

#[test]
fn kle_release_gate_requires_the_current_ast_workspace_boundary_test() -> Result<(), String> {
    let lines = crate::release_gate_sources::JUSTFILE.replace(
        " ast_linter_workspace_rules --",
        " ast_linter_removed_boundary_test --",
    );
    let lines = lines.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_kle_release_gate_from_lines(&lines);
    assert_error_contains(result, "ast_linter_workspace_rules")
}

#[test]
fn kle_release_gate_rejects_a_narrowed_ast_linter_command() -> Result<(), String> {
    let lines = crate::release_gate_sources::JUSTFILE.replace(
        " ast_linter -- --nocapture",
        " ast_linter ast_linter_kal_standard_rules_are_clean -- --nocapture",
    );
    let lines = lines.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_kle_release_gate_from_lines(&lines);
    assert_error_contains(result, "ast_linter -- --nocapture")
}

#[test]
fn release_check_runs_kle_verification_after_cleanup_definition() -> Result<(), String> {
    ReleaseGateAudit::validate_release_check_ordering()
}

#[test]
fn release_cleanup_only_cleans_cargo_tagged_targets() -> Result<(), String> {
    let justfile = crate::release_gate_sources::JUSTFILE;
    if !justfile.contains("release-check-clean-generated-artifacts:")
        || !justfile.contains("CACHEDIR.TAG")
        || !justfile.contains("Signature: 8a477f597d28d172789f06886806bc55")
    {
        return Err("release cleanup must verify Cargo's CACHEDIR.TAG signature".to_string());
    }
    if justfile.contains("{{CARGO}} clean --target-dir tools/kle-storybook/target") {
        return Err(
            "release cleanup must not unconditionally clean a target directory".to_string(),
        );
    }
    Ok(())
}

#[test]
fn release_workflow_runs_post_publish_completion_audits() -> Result<(), String> {
    ReleaseGateAudit::validate_release_workflow_ordering()
}

#[test]
fn release_workflow_manual_guard_fails_when_publish_false_can_continue() -> Result<(), String> {
    let lines = [
        "      publish_crates:",
        "        default: false",
        "      - name: Reject partial manual release",
        "        if: github.event_name == 'workflow_dispatch'",
        "      - name: Release check",
    ];
    let result = ReleaseGateAudit::validate_release_workflow_manual_guard_from_lines(&lines);
    assert_error_contains(result, "default")
}

#[test]
fn source_closure_native_host_contract_is_present() -> Result<(), String> {
    ReleaseGateAudit::validate_source_closure_native_host_contract()
}

#[test]
fn source_closure_native_host_contract_rejects_unsupported_materialize_option() -> Result<(), String>
{
    let lines = source_closure_workflow_fixture()
        .replace("--canonical-root artifacts", "--artifact-dir artifacts");
    let lines = lines.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_source_closure_native_host_contract_from_lines(&lines);
    assert_error_contains(result, "--canonical-root artifacts")
}

#[test]
fn source_closure_materialize_recipe_rejects_unsupported_option() -> Result<(), String> {
    let lines = crate::release_gate_sources::JUSTFILE.replace(
        "--canonical-root \"artifacts\"",
        "--artifact-dir \"artifacts\"",
    );
    let lines = lines.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_source_closure_materialize_recipe_from_lines(&lines);
    assert_error_contains(result, "--canonical-root \"artifacts\"")
}

#[test]
fn source_closure_native_host_contract_rejects_push_execution() -> Result<(), String> {
    let lines = source_closure_workflow_fixture().replace(
        "if: github.event_name == 'workflow_dispatch' && github.ref_protected",
        "if: github.event_name == 'push'",
    );
    let lines = lines.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_source_closure_native_host_contract_from_lines(&lines);
    assert_error_contains(result, "workflow_dispatch")
}

#[test]
fn source_closure_native_host_contract_rejects_raw_control_env() -> Result<(), String> {
    let lines = source_closure_workflow_fixture().replace(
        "run: >-",
        "env:\n          KATANA_OPEN_WORKSPACE_PATH: /tmp/workspace\n        run: >-",
    );
    let lines = lines.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_source_closure_native_host_contract_from_lines(&lines);
    assert_error_contains(result, "forbidden")
}

#[test]
fn source_closure_native_host_contract_rejects_fallback() -> Result<(), String> {
    let lines = source_closure_workflow_fixture().replace(
        "--test physical_open_workspace -- --test-threads=1",
        "--test physical_open_workspace -- --test-threads=1 || true",
    );
    let lines = lines.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_source_closure_native_host_contract_from_lines(&lines);
    assert_error_contains(result, "forbidden")
}

#[test]
fn source_closure_native_host_contract_rejects_missing_full_editor_parity_gate()
-> Result<(), String> {
    let lines = source_closure_workflow_fixture()
        .replace("KATANA_REPO=../katana just full-parity-check", "true");
    let lines = lines.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_source_closure_native_host_contract_from_lines(&lines);
    assert_error_contains(result, "full KatanA editor parity gate")
}

#[test]
fn source_closure_native_host_contract_rejects_native_job_push_trigger() -> Result<(), String> {
    let lines = source_closure_workflow_fixture().replace(
        "    steps:\n      - name: Checkout KLE",
        "    push:\n      branches: [master]\n    steps:\n      - name: Checkout KLE",
    );
    let lines = lines.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_source_closure_native_host_contract_from_lines(&lines);
    assert_error_contains(result, "protected workflow_dispatch-only")
}

#[test]
fn source_closure_native_host_contract_rejects_native_job_environment_input() -> Result<(), String>
{
    let lines = source_closure_workflow_fixture().replace(
        "    steps:\n      - name: Checkout KLE",
        "    env:\n      WORKSPACE_PATH: /tmp/workspace\n    steps:\n      - name: Checkout KLE",
    );
    let lines = lines.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_source_closure_native_host_contract_from_lines(&lines);
    assert_error_contains(result, "environment-variable inputs")
}

#[test]
fn source_closure_native_host_contract_rejects_duplicate_native_job() -> Result<(), String> {
    let workflow = source_closure_workflow_fixture();
    let duplicate = workflow
        .find("  native-host-e2e:")
        .map(|index| workflow[index..].to_string())
        .ok_or_else(|| "fixture native job missing".to_string())?;
    let lines = format!("{workflow}\n{duplicate}");
    let lines = lines.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_source_closure_native_host_contract_from_lines(&lines);
    assert_error_contains(result, "exactly one native-host-e2e")
}

fn source_closure_workflow_fixture() -> String {
    crate::release_gate_sources::SOURCE_CLOSURE_WORKFLOW.to_string()
}

#[test]
fn release_dependency_graph_includes_expected_tasks() -> Result<(), String> {
    ReleaseGateAudit::validate_release_dependency_graph_ordering()
}

#[test]
fn pre_push_gate_keeps_all_local_checks_and_defers_only_three_os_artifacts() -> Result<(), String> {
    ReleaseGateAudit::validate_pre_push_gate()
}

#[test]
fn release_preflight_keeps_local_release_gates_without_source_closure_artifacts()
-> Result<(), String> {
    let justfile = crate::release_gate_sources::JUSTFILE;
    let required = "release-preflight-check: release-target-check pre-push-check coverage";
    if !justfile.contains(required)
        || !justfile.contains("bash scripts/release/assert-crates-not-published.sh \"{{VERSION}}\"")
    {
        return Err("release preflight must retain target, local, coverage, package, and unpublished-crate gates".into());
    }
    let workflow = include_str!("../../../.github/workflows/release-preflight.yml");
    if !workflow.contains("run: just pre-push-check")
        || !workflow.contains("release-preflight-check")
        || workflow.contains("run: just check")
    {
        return Err("release preflight must defer only source-closure artifact validation".into());
    }
    Ok(())
}

#[test]
fn release_completion_audit_is_read_only() -> Result<(), String> {
    ReleaseGateAudit::validate_completion_audit_is_read_only()
}

#[test]
fn release_docs_preserve_completion_boundary() -> Result<(), String> {
    ReleaseGateAudit::validate_release_docs_completion_boundary()
}

#[test]
fn release_dependency_graph_fails_when_release_check_depends_on_release_verify()
-> Result<(), String> {
    let lines = [
        "katana-repo-adapter-check:",
        "check: fmt-check check-types",
        "release-verify: check coverage",
        "release-check: release-target-check release-verify release-check-clean-generated-artifacts",
    ];
    let result = ReleaseGateAudit::validate_release_dependency_graph_from_lines(&lines);
    assert_error_contains(result, "not as a dependency")
}

#[test]
fn release_dependency_graph_rejects_downstream_katana_adapter_dependency() -> Result<(), String> {
    let lines = [
        "katana-repo-adapter-check:",
        "check: fmt-check check-types kuc-contract-check katana-interface-check katana-downstream-check storybook-motion-artifact-gate katana-parity-check",
        "release-verify: check coverage",
        "release-check: release-target-check release-check-clean-generated-artifacts katana-repo-adapter-check",
    ];
    let result = ReleaseGateAudit::validate_release_dependency_graph_from_lines(&lines);
    assert_error_contains(result, "downstream KatanA adapter")
}

#[test]
fn release_check_rejects_downstream_adapter_invocation() -> Result<(), String> {
    let lines = [
        "release-check: release-target-check release-check-clean-generated-artifacts",
        "    just release-verify",
        "    just KATANA_REPO=\"{{KATANA_REPO}}\" katana-repo-adapter-check",
    ];
    let result = ReleaseGateAudit::validate_release_check_ordering_from_lines(&lines);
    assert_error_contains(result, "downstream KatanA adapter")
}

#[test]
fn release_dependency_graph_fails_when_release_verify_lacks_check_or_coverage() -> Result<(), String>
{
    let lines = [
        "katana-repo-adapter-check:",
        "check: fmt-check check-types kuc-contract-check katana-interface-check katana-downstream-check storybook-motion-artifact-gate katana-parity-check",
        "release-verify: check",
        "release-check: release-target-check release-check-clean-generated-artifacts",
    ];
    let result = ReleaseGateAudit::validate_release_dependency_graph_from_lines(&lines);
    assert_error_contains(result, "coverage")
}

#[test]
fn release_dependency_graph_fails_when_check_lacks_required_kle_release_tasks() -> Result<(), String>
{
    let lines = [
        "katana-repo-adapter-check:",
        "check: fmt-check check-types lint unit-test",
        "release-verify: check coverage",
        "release-check: release-target-check release-check-clean-generated-artifacts",
    ];
    let result = ReleaseGateAudit::validate_release_dependency_graph_from_lines(&lines);
    assert_error_contains(result, "kuc-contract-check")
}

#[test]
fn release_completion_audit_read_only_fails_on_mutating_commands() -> Result<(), String> {
    let lines = [
        "gh issue list --repo owner/repo --state open",
        "gh issue edit 1 --repo owner/repo --body-file bad.md",
    ];
    let result = ReleaseGateAudit::validate_completion_audit_read_only_from_lines(&lines);
    assert_error_contains(result, "read-only")
}
