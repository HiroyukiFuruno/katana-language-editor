use super::{
    ArtifactValidationMode, FixtureBuilder, SourceClosureArtifact, SourceClosureArtifactValidator,
    TestResult, complete_artifacts, read_artifact, validation_error_message, write_file,
};
use crate::source_closure::edge_model::ExternalUiDependency;

fn incomplete_dependency() -> Result<ExternalUiDependency, serde_json::Error> {
    serde_json::from_value(serde_json::json!({
        "package": "egui", "version": "0.36.1", "source": "registry+fixture",
        "package_checksum": "fixture", "lock_sha256": "fixture",
        "source_files": [{"path": "egui/src/lib.rs", "sha256": "fixture"}],
        "symbols": ["TextEdit::multiline"],
        "symbol_spans": [{"symbol": "TextEdit::multiline", "span": "egui:src/lib.rs:1:0-2:0"}],
        "katana_invoking_edges": [{
            "kind": "call", "source_file": "src/editor.rs", "span": "1:0-2:0",
            "katana_symbol": "editor", "dependency_symbol": "TextEdit::multiline"
        }],
        "direct_semantic_edges": [],
        "replacement_leafs": [], "complete": false,
        "unresolved_evidence": ["transitive semantic closure has not been captured"],
        "fingerprint": "fixture"
    }))
}

fn assert_all_modes_reject(dependencies: Vec<ExternalUiDependency>, expected: &str) -> TestResult {
    let fixture = FixtureBuilder::root()?;
    let paths = complete_artifacts(fixture.path())?;
    let mut source: SourceClosureArtifact = read_artifact(&paths.source_closure)?;
    source.external_ui_semantic_dependencies = dependencies;
    write_file(&paths.source_closure, &source)?;
    for mode in [
        ArtifactValidationMode::Rc,
        ArtifactValidationMode::KleRelease,
        ArtifactValidationMode::Full,
    ] {
        let result = SourceClosureArtifactValidator::validate_for_mode(&paths, mode);
        let error = validation_error_message(result, "expected external closure rejection")?;
        assert!(error.contains(expected), "{error}");
    }
    Ok(())
}

#[test]
fn every_validator_mode_rejects_missing_external_dependency() -> TestResult {
    assert_all_modes_reject(Vec::new(), "external UI dependency egui is missing")
}

#[test]
fn every_validator_mode_rejects_incomplete_external_dependency() -> TestResult {
    assert_all_modes_reject(
        vec![incomplete_dependency()?],
        "external UI dependency egui is incomplete",
    )
}

#[test]
fn declaring_complete_does_not_hide_unresolved_external_evidence() -> TestResult {
    let mut dependency = incomplete_dependency()?;
    dependency.complete = true;
    assert_all_modes_reject(
        vec![dependency],
        "external UI dependency egui has unresolved evidence",
    )
}
