use crate::release_gate_sources::JUSTFILE;

fn validate(justfile: &str) -> Result<(), String> {
    crate::release_gate_hook::ReleaseGateHookAudit::validate_pre_push_recipe_body(
        &justfile.lines().collect::<Vec<_>>(),
    )
}

#[test]
fn production_justfile_pre_push_postcheck_passes() -> Result<(), String> {
    validate(JUSTFILE)
}

#[test]
fn rejects_removed_postcheck() {
    for required in [
        "    python3 -B scripts/governance/cli.py verify-clean-worktree\n",
        "    @echo \"pre-push checks passed\"\n",
    ] {
        assert!(JUSTFILE.contains(required));
        assert!(validate(&JUSTFILE.replace(required, "")).is_err());
    }
}

#[test]
fn rejects_recipe_body_under_another_header() {
    let justfile = JUSTFILE.replace(
        "pre-push-check: governance-test fmt-check check-types lint unit-test ast-lint kuc-contract-check katana-interface-check katana-downstream-check storybook-motion-artifact-gate\n    python3 -B scripts/governance/cli.py verify-clean-worktree\n    @echo \"pre-push checks passed\"\n",
        "other-recipe:\n    python3 -B scripts/governance/cli.py verify-clean-worktree\n    @echo \"pre-push checks passed\"\n",
    );
    assert!(validate(&justfile).is_err());
}

#[test]
fn rejects_early_exit_before_worktree_check() {
    let justfile = JUSTFILE.replace(
        "    python3 -B scripts/governance/cli.py verify-clean-worktree",
        "    exit 0\n    python3 -B scripts/governance/cli.py verify-clean-worktree",
    );
    assert!(validate(&justfile).is_err());
}

#[test]
fn rejects_reversed_check_and_postcheck_order() {
    let justfile = JUSTFILE.replace(
        "    python3 -B scripts/governance/cli.py verify-clean-worktree\n    @echo \"pre-push checks passed\"",
        "    @echo \"pre-push checks passed\"\n    python3 -B scripts/governance/cli.py verify-clean-worktree",
    );
    assert!(validate(&justfile).is_err());
}

#[test]
fn rejects_duplicate_pre_push_check_headers() {
    let justfile = format!("{JUSTFILE}\npre-push-check:\n    @echo duplicate\n");
    assert!(validate(&justfile).is_err());
}
