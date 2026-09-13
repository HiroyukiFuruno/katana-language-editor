use super::capability_manifest_test_fixtures::FixtureBuilder;
use crate::{
    capability_manifest::{
        CapabilityManifest, CapabilityManifestAudit, CapabilityOwner, HostEffectKind,
        KatanaSourceEvidence, KleActualFrameHarness, KleActualInputEvidence, KleHostE2eEvidence,
        KleHostE2eTestLocator, KleSourceLocator,
    },
    matrix::{EvidenceClassification, FeatureVerification},
};

const OWNERS: &[CapabilityOwner] = &[
    CapabilityOwner::KucRuntime,
    CapabilityOwner::KleBinding,
    CapabilityOwner::KatanaHost,
];
const SOURCE: &[KatanaSourceEvidence] = &[KatanaSourceEvidence {
    path: "src/editor.rs",
    line: 1,
    marker: "source marker",
}];
const HOST_E2E: KleHostE2eEvidence = KleHostE2eEvidence {
    test: KleHostE2eTestLocator {
        target: "actual_host",
        source_path: "tools/katana-host-e2e/tests/actual_host.rs",
        selector: "actual_kle_context_menu_authoring_raw_input_changes_katana_document",
    },
    effect: HostEffectKind::ContextAuthoring,
};
const HARNESS: KleActualFrameHarness = KleActualFrameHarness {
    public_show_callsite: KleSourceLocator {
        source_path: "tests/actual_frame_harness.rs",
        line: 2,
        marker: "context.run_ui(raw_input(events), |ui| result = Some(editor.show(ui)))",
    },
    raw_input_root: KleSourceLocator {
        source_path: "tests/actual_frame_harness.rs",
        line: 4,
        marker: "fn raw_input(events: Vec<Event>) -> RawInput {",
    },
    raw_input_construction: KleSourceLocator {
        source_path: "tests/actual_frame_harness.rs",
        line: 5,
        marker: "RawInput {",
    },
    scenario_implementation: KleSourceLocator {
        source_path: "tests/actual_frame_harness.rs",
        line: 9,
        marker: "impl DirectTextSurfaceScenario {",
    },
    symbol: "DirectTextSurfaceScenario",
    call_path: "DirectTextSurfaceScenario::run",
};
const MANIFEST: CapabilityManifest = CapabilityManifest {
    classification: EvidenceClassification::ConfirmedKatana,
    katana_sources: SOURCE,
    owners: OWNERS,
    kle_actual_input: KleActualInputEvidence {
        feature_id: "fixture",
        source_path: "tests/actual_input.rs",
        selector: "public_show_fixture",
        harness: HARNESS,
    },
    host_e2e: HOST_E2E,
};

#[test]
fn rejects_source_marker_mismatch() -> Result<(), String> {
    let root = FixtureBuilder::root()?;
    FixtureBuilder::write_katana(
        root.path(),
        "editor_kle_downstream_adapter",
        "#[test]\nfn adapter_roundtrip() {}\n",
    )?;
    let bad_manifest = Box::leak(Box::new(CapabilityManifest {
        katana_sources: Box::leak(Box::new([KatanaSourceEvidence {
            marker: "other marker",
            ..SOURCE[0]
        }])),
        ..MANIFEST
    }));
    let error = CapabilityManifestAudit::validate(&feature(bad_manifest), root.path())
        .err()
        .ok_or("expected failure")?;
    error
        .contains("KatanA source marker mismatch")
        .then_some(())
        .ok_or(error)
}

#[test]
fn rejects_incomplete_host_e2e_declaration() -> Result<(), String> {
    let manifest = Box::leak(Box::new(CapabilityManifest {
        host_e2e: KleHostE2eEvidence {
            test: KleHostE2eTestLocator {
                selector: "",
                ..HOST_E2E.test
            },
            ..HOST_E2E
        },
        ..MANIFEST
    }));
    match CapabilityManifestAudit::validate_feature_set(&[feature(manifest)]) {
        Err(error) if error.contains("incomplete KLE host E2E evidence") => Ok(()),
        Ok(()) => Err("incomplete host E2E evidence was accepted".to_string()),
        Err(error) => Err(format!("unexpected rejection: {error}")),
    }
}

#[test]
fn rejects_reused_selector() -> Result<(), String> {
    let first = feature(&MANIFEST);
    let manifest = Box::leak(Box::new(CapabilityManifest {
        kle_actual_input: KleActualInputEvidence {
            feature_id: "fixture-two",
            ..MANIFEST.kle_actual_input
        },
        ..MANIFEST
    }));
    let second = FeatureVerification {
        id: "fixture-two",
        name: "fixture two",
        automated_checks: &["test"],
        remaining_gap: "required",
        manifest,
    };
    let error = CapabilityManifestAudit::validate_feature_set(&[first, second])
        .err()
        .ok_or("expected failure")?;
    error
        .contains("KLE actual-input selector is reused")
        .then_some(())
        .ok_or(error)
}

fn feature(manifest: &'static CapabilityManifest) -> FeatureVerification {
    FeatureVerification {
        id: "fixture",
        name: "fixture",
        automated_checks: &["test"],
        remaining_gap: "required",
        manifest,
    }
}
