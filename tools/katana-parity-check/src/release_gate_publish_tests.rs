use crate::release_gate::ReleaseGateAudit;

fn assert_error_contains(result: Result<(), String>, expected: &str) -> Result<(), String> {
    match result {
        Ok(()) => Err(format!("expected error containing {expected}, got Ok")),
        Err(error) if error.contains(expected) => Ok(()),
        Err(error) => Err(format!("expected error containing {expected}, got {error}")),
    }
}

#[test]
fn publish_crates_waits_for_egui_visibility() -> Result<(), String> {
    ReleaseGateAudit::validate_publish_crate_visibility_ordering()
}

#[test]
fn publish_crates_requires_kuc_release_set_before_kle_publish() -> Result<(), String> {
    ReleaseGateAudit::validate_kuc_release_set_gate_from_lines(
        &crate::release_gate_sources::PUBLISH_CRATES
            .lines()
            .collect::<Vec<_>>(),
    )
}

#[test]
fn publish_crates_rejects_omitted_kuc_target() -> Result<(), String> {
    let source =
        crate::release_gate_sources::PUBLISH_CRATES.replace("katana-ui-core", "not-a-kuc-crate");
    let lines = source.lines().collect::<Vec<_>>();
    assert_error_contains(
        ReleaseGateAudit::validate_kuc_release_set_gate_from_lines(&lines),
        "katana-ui-core",
    )
}

#[test]
fn publish_crates_rejects_omitted_kuc_version() -> Result<(), String> {
    let source =
        crate::release_gate_sources::PUBLISH_CRATES.replace("${package}@${version}", "${package}");
    let lines = source.lines().collect::<Vec<_>>();
    assert_error_contains(
        ReleaseGateAudit::validate_kuc_release_set_gate_from_lines(&lines),
        "${package}@${version}",
    )
}

#[test]
fn publish_crates_rejects_kuc_gate_after_publish() -> Result<(), String> {
    let source = crate::release_gate_sources::PUBLISH_CRATES;
    let gate = source
        .find("require_kuc_release_set\n")
        .ok_or_else(|| "KUC gate missing from fixture".to_string())?;
    let gate_end = gate + "require_kuc_release_set\n".len();
    let reordered = format!(
        "{}{}{}",
        &source[..gate],
        &source[gate_end..],
        &source[gate..gate_end]
    );
    let lines = reordered.lines().collect::<Vec<_>>();
    assert_error_contains(
        ReleaseGateAudit::validate_kuc_release_set_gate_from_lines(&lines),
        "must precede",
    )
}

#[test]
fn publish_crates_ordering_fails_without_egui_visibility_wait() -> Result<(), String> {
    let lines = [
        "require_kuc_release_set",
        "workspace_manifest=\"${PWD}/Cargo.toml\"",
        "tomllib",
        "katana-ui-core",
        "dependency_name",
        "version",
        "egui",
        "text-raster",
        "cargo info \"${package}@${version}\" --registry crates-io",
        "publish_if_needed katana-language-editor \"${CARGO_REGISTRY_TOKEN}\"",
        "wait_for_crate katana-language-editor",
        "publish_if_needed katana-language-editor-egui \"${CARGO_REGISTRY_TOKEN}\"",
    ];
    let result = ReleaseGateAudit::validate_publish_crate_visibility_from_lines(&lines);
    assert_error_contains(result, "katana-language-editor-egui")
}
