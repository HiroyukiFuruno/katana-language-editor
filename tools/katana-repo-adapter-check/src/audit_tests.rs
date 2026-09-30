use crate::{
    audit::KatanaRepoAdapterAudit,
    audit_runner::{KatanaRunnableAudit, SourceSnapshot},
    audit_tests_fixture::RepoFixture,
};

const BASELINE_TEST: &str = "editor_ui::test_integration_editor_line_numbers_visibility";
const DOWNSTREAM_TEST: &str =
    "editor_kle_downstream_adapter::kle_event_action_stream_updates_real_katana_editor_state";

#[test]
fn missing_dependency_reports_every_dependency() -> Result<(), String> {
    let fixture = RepoFixture::create("missing-dependency")?;
    fixture.write_complete(false, true, true)?;
    let error = match KatanaRepoAdapterAudit::new(fixture.config()).validate() {
        Ok(()) => return Err("expected missing dependencies to fail validation".to_string()),
        Err(error) => error,
    };
    assert!(error.contains("katana-language-editor ="));
    assert!(error.contains("katana-language-editor-egui ="));
    Ok(())
}

#[test]
fn missing_module_source_and_target_are_all_reported() -> Result<(), String> {
    let fixture = RepoFixture::create("missing-pieces")?;
    fixture.write_complete(true, false, false)?;
    let error = match KatanaRepoAdapterAudit::new(fixture.config()).validate() {
        Ok(()) => return Err("expected missing pieces to fail validation".to_string()),
        Err(error) => error,
    };
    assert!(error.contains("ui integration wrapper is missing"));
    assert!(error.contains("missing KLE downstream adapter source"));
    Ok(())
}

#[test]
fn forbidden_planned_adapter_marker_fails() -> Result<(), String> {
    let fixture = RepoFixture::create("forbidden-adapter-marker")?;
    fixture.write_complete(true, true, true)?;
    fixture.write_adapter_with_forbidden_marker()?;
    let error = match KatanaRepoAdapterAudit::new(fixture.config()).validate() {
        Ok(()) => return Err("expected forbidden marker to fail validation".to_string()),
        Err(error) => error,
    };
    assert!(error.contains("forbidden planned-patch marker"));
    Ok(())
}

#[test]
fn missing_linux_compose_mount_fails() -> Result<(), String> {
    let fixture = RepoFixture::create("missing-linux-compose-mount")?;
    fixture.write_complete(true, true, true)?;
    fixture.write(
        "platforms/linux/ci/compose.yml",
        "services:\n  ubuntu-test: {}\n",
    )?;
    assert_missing(
        &fixture,
        "check-linux compose mount",
        "katana-language-editor:ro",
    )
}

#[test]
fn missing_windows_compose_mount_fails() -> Result<(), String> {
    let fixture = RepoFixture::create("missing-windows-compose-mount")?;
    fixture.write_complete(true, true, true)?;
    fixture.write(
        "platforms/windows/ci/compose.yml",
        "services:\n  windows-test: {}\n",
    )?;
    assert_missing(
        &fixture,
        "check-windows compose mount",
        "katana-language-editor:ro",
    )
}

#[test]
fn missing_test_and_build_kle_clone_ref_fails() -> Result<(), String> {
    assert_missing_workflow("test-and-build.yml", "workflow test-and-build clone")
}

#[test]
fn missing_release_readiness_kle_clone_ref_fails() -> Result<(), String> {
    assert_missing_workflow("release-readiness.yml", "workflow release-readiness clone")
}

#[test]
fn missing_build_and_release_kle_clone_ref_fails() -> Result<(), String> {
    assert_missing_workflow("build-and-release.yml", "workflow build-and-release clone")
}

#[test]
fn complete_fixture_executes_tests_without_writing_source() -> Result<(), String> {
    let fixture = RepoFixture::create("complete")?;
    fixture.write_complete(true, true, true)?;
    let before = SourceSnapshot::capture(fixture.root())?;
    KatanaRepoAdapterAudit::new(fixture.config()).validate()?;
    assert_eq!(before, SourceSnapshot::capture(fixture.root())?);
    Ok(())
}

#[test]
fn source_snapshot_change_after_runnable_test_fails() -> Result<(), String> {
    let fixture = RepoFixture::create("source-snapshot-change")?;
    fixture.write_complete(true, true, true)?;
    fixture.write_adapter_that_mutates_source()?;
    let error = match KatanaRunnableAudit::new(fixture.root())
        .validate(&[BASELINE_TEST, DOWNSTREAM_TEST])
    {
        Ok(()) => return Err("expected source mutation to fail validation".to_string()),
        Err(error) => error,
    };
    assert!(error.contains("KatanA reference checkout changed during read-only validation"));
    Ok(())
}

fn assert_missing(fixture: &RepoFixture, label: &str, evidence: &str) -> Result<(), String> {
    let error = match KatanaRepoAdapterAudit::new(fixture.config()).validate() {
        Ok(()) => return Err(format!("expected {label} to fail validation")),
        Err(error) => error,
    };
    assert!(error.contains(label));
    assert!(error.contains(evidence));
    Ok(())
}

fn assert_missing_workflow(path: &str, label: &str) -> Result<(), String> {
    let fixture = RepoFixture::create(label)?;
    fixture.write_complete(true, true, true)?;
    fixture.write(
        &format!(".github/workflows/{path}"),
        "name: workflow\nsteps: []\n",
    )?;
    assert_missing(&fixture, label, "KLE_REF")
}
