use std::{fs, path::Path};

use crate::capability_manifest::{
    HostE2eEvidenceValidator, HostEffectKind, KleHostE2eEvidence, KleHostE2eTestLocator,
};

const HOST_E2E_MANIFEST: &str = "tools/katana-host-e2e/Cargo.toml";
const HOST_E2E_TEST: &str = "tools/katana-host-e2e/tests/actual_host.rs";
const HOST_RUNTIME: &str = "tools/katana-host-e2e/src/host.rs";

#[test]
fn rejects_action_equality_without_a_real_ui_frame_or_document_effect() -> Result<(), String> {
    let root = fixture_root("equality-only");
    write_fixture(
        &root,
        "#[test]\nfn actual_kle_fixture() { let _ = KatanaHost::run_editor_request; assert_eq!(true, true); }",
    )?;
    let result = HostE2eEvidenceValidator::validate(&root, &fixture_evidence());
    assert_rejected(result, "KatanaApp")?;
    fs::remove_dir_all(root).map_err(|error| format!("remove fixture: {error}"))
}

#[test]
fn accepts_real_host_frame_and_concrete_document_effect() -> Result<(), String> {
    let root = fixture_root("actual-effect");
    write_fixture(
        &root,
        "#[test]\nfn actual_kle_fixture() { helper(); }\nfn helper() { KatanaHost::run_editor_request(); let _ = core::mem::size_of::<katana_ui::shell::KatanaApp>(); let authoring_specs = vec![()]; if authoring_specs.len() != 30 {} if document.buffer != expected.buffer || !document.is_dirty {} ContextMenuPhysicalInput::case_specs(); }",
    )?;
    HostE2eEvidenceValidator::validate(&root, &fixture_evidence())
        .map_err(|error| format!("real host fixture was rejected: {error}"))?;
    fs::remove_dir_all(root).map_err(|error| format!("remove fixture: {error}"))
}

fn fixture_root(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "katana-parity-host-e2e-{}-{label}",
        std::process::id()
    ))
}

fn write_fixture(root: &Path, test_source: &str) -> Result<(), String> {
    fs::create_dir_all(root.join("tools/katana-host-e2e/tests"))
        .map_err(|error| format!("create fixture tests directory: {error}"))?;
    fs::create_dir_all(root.join("tools/katana-host-e2e/src"))
        .map_err(|error| format!("create fixture source directory: {error}"))?;
    fs::write(
        root.join(HOST_E2E_MANIFEST),
        "[package]\nname = \"katana-host-e2e\"\n",
    )
    .map_err(|error| format!("write fixture manifest: {error}"))?;
    fs::write(root.join(HOST_E2E_TEST), test_source)
        .map_err(|error| format!("write fixture test: {error}"))?;
    fs::write(
        root.join(HOST_RUNTIME),
        "impl KatanaHost { fn run_editor_request_with_context() { app.trigger_action(action); Self::run_ui_frame_with_context(); } }",
    )
    .map_err(|error| format!("write fixture runtime: {error}"))?;
    Ok(())
}

fn assert_rejected(result: Result<(), String>, expected: &str) -> Result<(), String> {
    match result {
        Err(error) if error.contains(expected) => Ok(()),
        Ok(()) => Err(format!("expected rejection containing {expected:?}")),
        Err(error) => Err(format!("unexpected rejection: {error}")),
    }
}

const fn fixture_evidence() -> KleHostE2eEvidence {
    KleHostE2eEvidence {
        test: KleHostE2eTestLocator {
            target: "actual_host",
            source_path: HOST_E2E_TEST,
            selector: "actual_kle_fixture",
        },
        effect: HostEffectKind::ContextAuthoring,
    }
}
