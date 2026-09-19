#[test]
fn release_preflight_keeps_local_release_gates_without_source_closure_artifacts()
-> Result<(), String> {
    let justfile = crate::release_gate_sources::JUSTFILE;
    let required = "release-preflight-check: release-target-check pre-push-check coverage";
    if !justfile.contains(required)
        || !justfile.contains("bash scripts/release/assert-crates-not-published.sh \"{{VERSION}}\"")
    {
        return Err(
            "release preflight must retain target, local, coverage, package, and unpublished-crate gates"
                .into(),
        );
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
fn release_workflows_checkout_the_fixed_katana_reference_for_source_bound_tests()
-> Result<(), String> {
    for (name, workflow) in [
        (
            "release preflight",
            include_str!("../../../.github/workflows/release-preflight.yml"),
        ),
        ("release", crate::release_gate_sources::RELEASE_WORKFLOW),
    ] {
        for required in [
            "- name: Checkout fixed KatanA reference",
            "repository: HiroyukiFuruno/KatanA",
            "ref: 4f6a6287c650a38633c7baeb544a92e739c68567",
            "path: katana",
        ] {
            if !workflow.contains(required) {
                return Err(format!(
                    "{name} workflow is missing fixed KatanA checkout `{required}`"
                ));
            }
        }
    }
    Ok(())
}
