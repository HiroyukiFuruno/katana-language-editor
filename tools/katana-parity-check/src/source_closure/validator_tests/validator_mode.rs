use super::{
    ArtifactValidationMode, BranchCatalogArtifact, ExecutionRecordArtifact, FixtureBuilder,
    KatanaHostE2eState, LeafManifestArtifact, SourceClosureArtifact,
    SourceClosureArtifactValidator, TestResult, complete_artifacts, read_artifact, unique_temp_dir,
    validation_error_message, write_file,
};

#[test]
fn rejects_legacy_leafs_that_do_not_match_canonical_projection() -> TestResult {
    let dir = unique_temp_dir("katana-parity-source-closure-validator")?;
    let paths = complete_artifacts(&dir)?;
    let result =
        SourceClosureArtifactValidator::validate_for_mode(&paths, ArtifactValidationMode::Full);
    let message = validation_error_message(result, "expected canonical projection rejection")?;
    assert!(message.contains("leaf has no structured candidate provenance"));
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn accepts_complete_canonical_bundle_before_release_closure_checks() -> TestResult {
    let dir = unique_temp_dir("katana-parity-source-closure-canonical-integrity")?;
    let paths = complete_artifacts(&dir)?;
    let source: SourceClosureArtifact = read_artifact(&paths.source_closure)?;
    let branches: BranchCatalogArtifact = read_artifact(&paths.branch_catalog)?;
    let leaf: LeafManifestArtifact = read_artifact(&paths.leaf_manifest)?;
    leaf.canonical_binding.validate(&source, &branches)?;
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn release_modes_reject_retained_unresolved_canonical_population() -> TestResult {
    let dir = unique_temp_dir("katana-parity-source-closure-release-closure")?;
    let paths = complete_artifacts(&dir)?;
    for mode in [
        ArtifactValidationMode::KleRelease,
        ArtifactValidationMode::Full,
    ] {
        let result = SourceClosureArtifactValidator::validate_for_mode(&paths, mode);
        let message = validation_error_message(result, "expected unresolved closure rejection")?;
        assert!(
            message.contains("canonical binding has unresolved inventory rows"),
            "{message}"
        );
        assert!(
            message.contains("canonical binding has unresolved or unmatched T3 population"),
            "{message}"
        );
        assert!(
            message.contains("canonical binding has unresolved structured candidate population"),
            "{message}"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn default_validator_requires_kle_release_candidate() -> TestResult {
    let fixture = FixtureBuilder::root()?;
    let paths = complete_artifacts(fixture.path())?;
    let result = SourceClosureArtifactValidator::validate(&paths);
    let message = validation_error_message(result, "expected missing KLE release candidate")?;
    assert!(message.contains("KLE release evidence is required"));
    Ok(())
}

#[test]
fn rc_still_rejects_legacy_leafs_after_downstream_host_claim_is_removed() -> TestResult {
    let dir = unique_temp_dir("katana-parity-source-closure-rc")?;
    let paths = complete_artifacts(&dir)?;
    let mut leaf_root: LeafManifestArtifact = read_artifact(&paths.leaf_manifest)?;
    leaf_root.leafs[0].host_e2e.state = KatanaHostE2eState::DownstreamRequired;
    leaf_root.leafs[0].execution_id = None;
    write_file(&paths.leaf_manifest, &leaf_root)?;
    let mut execution_root: ExecutionRecordArtifact = read_artifact(&paths.execution_record)?;
    execution_root.executions.clear();
    write_file(&paths.execution_record, &execution_root)?;
    let result =
        SourceClosureArtifactValidator::validate_for_mode(&paths, ArtifactValidationMode::Rc);
    let message = validation_error_message(result, "expected canonical projection rejection")?;
    assert!(message.contains("leaf has no structured candidate provenance"));
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn rejects_tampered_compiled_requirements_inventory_sha() -> TestResult {
    let dir = unique_temp_dir("katana-parity-source-closure-inventory-sha")?;
    let paths = complete_artifacts(&dir)?;
    let mut leaf_root: LeafManifestArtifact = read_artifact(&paths.leaf_manifest)?;
    leaf_root.canonical_binding.inventory.requirements_sha256 = "tampered".to_string();
    write_file(&paths.leaf_manifest, &leaf_root)?;

    let result =
        SourceClosureArtifactValidator::validate_for_mode(&paths, ArtifactValidationMode::Rc);
    let message = validation_error_message(result, "expected inventory SHA rejection")?;
    assert!(message.contains("inventory differs from canonical reconstruction"));
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn rejects_added_structured_unresolved_population() -> TestResult {
    let dir = unique_temp_dir("katana-parity-source-closure-structured-unresolved")?;
    let paths = complete_artifacts(&dir)?;
    let mut leaf_root: LeafManifestArtifact = read_artifact(&paths.leaf_manifest)?;
    leaf_root
        .canonical_binding
        .structured
        .source_action_unresolved
        .push(
            super::super::super::source_action_binding::SourceActionUnresolved {
                kind: "tampered".to_string(),
                source_edge_kind: None,
                source_edge_detail: None,
                source_edge_resolution: None,
                action_variant: None,
                path: "fixture.rs".to_string(),
                span: None,
                reason: "must remain visible".to_string(),
            },
        );
    write_file(&paths.leaf_manifest, &leaf_root)?;

    let result =
        SourceClosureArtifactValidator::validate_for_mode(&paths, ArtifactValidationMode::Rc);
    let message = validation_error_message(result, "expected structured report rejection")?;
    assert!(message.contains("structured leaf candidates differs from canonical reconstruction"));
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn rejects_rehashed_binding_from_a_reduced_branch_population() -> TestResult {
    let fixture = FixtureBuilder::root()?;
    let paths = complete_artifacts(fixture.path())?;
    let source: SourceClosureArtifact = read_artifact(&paths.source_closure)?;
    let mut branches: BranchCatalogArtifact = read_artifact(&paths.branch_catalog)?;
    assert!(!branches.branches.is_empty());
    branches.branches.clear();
    let mut manifest: LeafManifestArtifact = read_artifact(&paths.leaf_manifest)?;
    manifest.canonical_binding =
        super::super::super::canonical_leaf_binding::CanonicalLeafBindingEvidence::build(
            &source,
            &branches,
            &super::super::super::scan_state::ScanState::default(),
        )?;
    manifest.canonical_binding.t2.validate_fingerprint()?;
    manifest.canonical_binding.t3.validate_fingerprint()?;
    write_file(&paths.leaf_manifest, &manifest)?;
    let result =
        SourceClosureArtifactValidator::validate_for_mode(&paths, ArtifactValidationMode::Rc);
    let message = validation_error_message(result, "expected reduced population rejection")?;
    assert!(message.contains("T2 requirement binding differs from canonical reconstruction"));
    Ok(())
}

#[test]
fn rejects_missing_required_binding_and_unknown_nested_fields() -> TestResult {
    let fixture = FixtureBuilder::root()?;
    let paths = complete_artifacts(fixture.path())?;
    let manifest: LeafManifestArtifact = read_artifact(&paths.leaf_manifest)?;
    let original = serde_json::to_value(&manifest)?;
    let mut missing = original.clone();
    missing
        .as_object_mut()
        .ok_or("expected manifest object")?
        .remove("canonical_binding");
    write_file(&paths.leaf_manifest, &missing)?;
    let error = validation_error_message(
        read_artifact::<LeafManifestArtifact>(&paths.leaf_manifest),
        "expected required canonical binding rejection",
    )?;
    assert!(
        error.contains("missing field `canonical_binding`"),
        "{error}"
    );

    for field in ["inventory", "t2", "t3", "structured"] {
        let mut invalid = original.clone();
        invalid["canonical_binding"][field]
            .as_object_mut()
            .ok_or("expected nested report object")?
            .insert("unknown_claim".into(), serde_json::json!(true));
        write_file(&paths.leaf_manifest, &invalid)?;
        let error = validation_error_message(
            read_artifact::<LeafManifestArtifact>(&paths.leaf_manifest),
            "expected unknown report field rejection",
        )?;
        assert!(error.contains("unknown field `unknown_claim`"), "{error}");
    }
    Ok(())
}
