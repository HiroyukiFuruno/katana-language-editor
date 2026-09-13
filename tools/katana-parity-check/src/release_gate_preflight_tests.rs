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
