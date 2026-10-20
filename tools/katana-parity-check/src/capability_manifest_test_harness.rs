use super::{
    capability_manifest_kle_evidence::KleEvidenceValidator,
    capability_manifest_test_fixtures::FixtureBuilder,
};
use crate::capability_manifest::{
    CapabilityManifestAudit, KleActualFrameHarness, KleActualInputEvidence, KleSourceLocator,
};

const HARNESS: KleActualFrameHarness = KleActualFrameHarness {
    public_show_callsite: KleSourceLocator {
        source_path: "tests/actual_input.rs",
        line: 1,
        marker: "host.show_public_api_with_opaque_frame(ui)",
    },
    raw_input_root: KleSourceLocator {
        source_path: "tests/actual_input.rs",
        line: 3,
        marker: "fn raw_input(events: Vec<Event>) -> RawInput {",
    },
    raw_input_construction: KleSourceLocator {
        source_path: "tests/actual_input.rs",
        line: 4,
        marker: "RawInput {",
    },
    scenario_implementation: KleSourceLocator {
        source_path: "tests/actual_input.rs",
        line: 7,
        marker: "impl StorybookHost {",
    },
    symbol: "StorybookHost",
    call_path: "StorybookHost::show_public_api_with_opaque_frame",
};
const INPUT: KleActualInputEvidence = KleActualInputEvidence {
    feature_id: "fixture",
    source_path: "tests/actual_input.rs",
    selector: "run_frame",
    harness: HARNESS,
};

#[test]
fn resolves_current_public_kle_root_harness_call() -> Result<(), String> {
    let root = FixtureBuilder::root()?;
    FixtureBuilder::write_kle(
        root.path(),
        "fn run_frame(host: &mut StorybookHost, input: RawInput) { host.show_public_api_with_opaque_frame(ui); }\n",
    )?;
    KleEvidenceValidator::validate_at(root.path(), &INPUT)
}

#[test]
fn rejects_indirect_simulator_harness_call() -> Result<(), String> {
    let root = FixtureBuilder::root()?;
    FixtureBuilder::write_kle(
        root.path(),
        "fn run_frame(host: &mut FakeHost, input: RawInput) { host.show_public_api_with_opaque_frame(ui); }\n",
    )?;
    let error = KleEvidenceValidator::validate_at(root.path(), &INPUT)
        .err()
        .ok_or("expected failure")?;
    error
        .contains("not typed as StorybookHost")
        .then_some(())
        .ok_or(error)
}

#[test]
fn rejects_function_pointer_as_harness_call() -> Result<(), String> {
    let root = FixtureBuilder::root()?;
    FixtureBuilder::write_kle(
        root.path(),
        "fn run_frame(host: &mut StorybookHost, input: RawInput) { host.show(ui); }\n",
    )?;
    let error = KleEvidenceValidator::validate_at(root.path(), &INPUT)
        .err()
        .ok_or("expected failure")?;
    error
        .contains("missing KLE actual-input harness call")
        .then_some(())
        .ok_or(error)
}

#[test]
fn description_reports_transitive_chain() {
    let description = CapabilityManifestAudit::describe_kle_actual_input(&INPUT);
    assert!(description.starts_with("unresolved KLE actual-input source:"));
}

#[test]
fn rejects_missing_public_root_call() -> Result<(), String> {
    let root = FixtureBuilder::root()?;
    FixtureBuilder::write_kle(
        root.path(),
        "fn run_frame(host: &mut StorybookHost, input: RawInput) { let _ = input; }\n",
    )?;
    let error = KleEvidenceValidator::validate_at(root.path(), &INPUT)
        .err()
        .ok_or("expected failure")?;
    error
        .contains("missing KLE actual-input harness call")
        .then_some(())
        .ok_or(error)
}
