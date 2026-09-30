use super::*;

#[test]
fn complete_three_profile_raw_fixture_verifies_and_materializes() -> TestResult {
    let fixture = fixture("valid")?;
    let verified = string_result(SourceClosureInputLoaderVerifier::load(&fixture.input_json))?;
    assert_eq!(verified.identity_fingerprint().len(), 71);
    let artifact = string_result(SourceClosureMaterializer::materialize(
        &verified,
        &fixture.katana_root,
        &fixture.root.join("artifacts"),
    ))?;
    let output: crate::source_closure::model::SourceClosureArtifact =
        serde_json::from_slice(&fs::read(artifact)?)?;
    assert_eq!(output.profiles.len(), 3);
    assert!(
        output
            .files
            .iter()
            .any(|file| file.path == "crates/katana-ui/src/widgets/toggle/mod.rs")
    );
    let mut edges = output
        .files
        .iter()
        .flat_map(|file| file.incoming_edges.iter());
    assert!(edges.any(|edge| { edge.kind == "module" && edge.to_path.as_deref().is_some() }));
    assert!(
        output.unresolved_edges.iter().any(|edge| {
            edge.kind == "use"
                && edge.to_path.is_none()
                && !edge.span.trim().is_empty()
                && edge.to_symbol.as_deref().is_some_and(|reason| {
                    reason.contains("glob import is intentionally unresolved")
                })
        }),
        "glob imports must remain explicit unresolved closure edges"
    );
    let branch_catalog_path = fixture.root.join("artifacts").join("branch-catalog.json");
    let branch_catalog: crate::source_closure::model::BranchCatalogArtifact =
        serde_json::from_slice(&fs::read(branch_catalog_path)?)?;
    assert!(
        branch_catalog
            .branches
            .iter()
            .all(|branch| branch.outcomes.is_empty())
    );
    assert_eq!(
        branch_catalog.unclassified_branch_ids.len(),
        branch_catalog.branches.len()
    );
    let action_origins_path = fixture.root.join("artifacts").join("action-origins.json");
    let action_origins: crate::source_closure::model::ActionOriginsArtifact =
        serde_json::from_slice(&fs::read(action_origins_path)?)?;
    assert!(action_origins.actions.is_empty());
    assert!(action_origins.unresolved_action_origins.is_empty());
    let leaf_manifest_path = fixture.root.join("artifacts").join("leaf-manifest.json");
    let leaf_manifest: crate::source_closure::model::LeafManifestArtifact =
        serde_json::from_slice(&fs::read(leaf_manifest_path)?)?;
    assert!(leaf_manifest.leafs.is_empty());
    cleanup_dir(fixture.root)?;
    Ok(())
}

#[test]
fn fixtures_share_one_canonical_fixed_reference_root() -> TestResult {
    let first = fixture("cache-first")?;
    let second = fixture("cache-second")?;
    assert_eq!(first.katana_root, second.katana_root);
    assert_eq!(
        first.katana_root,
        string_result(fixed_reference_root())?.canonicalize()?
    );
    cleanup_dir(first.root)?;
    cleanup_dir(second.root)?;
    Ok(())
}

#[test]
fn fixed_reference_cache_is_detached_at_fixed_revision_and_matches_source_bytes() -> TestResult {
    let fixture = fixture("cache-integrity")?;
    let root = string_result(fixed_reference_root())?;
    let head = crate::system::ProcessService::create_command("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "HEAD"])
        .output()?;
    if !head.status.success() {
        return Err(test_error(format!(
            "git HEAD failed: {}",
            String::from_utf8_lossy(&head.stderr).trim()
        )));
    }
    assert_eq!(
        String::from_utf8(head.stdout)?.trim(),
        FIXED_KATANA_REVISION
    );
    for relative in DEFAULT_TREE_PATHS {
        let cached = fs::read(root.join(relative))?;
        let captured = fs::read(
            fixture
                .root
                .join("input")
                .join("raw/provenance/katana-tree")
                .join(relative),
        )?;
        assert_eq!(cached, captured, "source bytes differ: {relative}");
    }
    cleanup_dir(fixture.root)?;
    assert!(root.is_dir(), "fixture cleanup removed fixed cache");
    Ok(())
}

#[test]
fn fixture_cleanup_does_not_modify_fixed_reference_cache() -> TestResult {
    let root = string_result(fixed_reference_root())?.to_path_buf();
    let relative = DEFAULT_TREE_PATHS[0];
    let before = fs::read(root.join(relative))?;
    let fixture = fixture("cache-cleanup")?;
    cleanup_dir(fixture.root)?;
    assert_eq!(fs::read(root.join(relative))?, before);
    assert!(
        root.join(".git").is_dir(),
        "fixed cache git metadata removed"
    );
    Ok(())
}

#[test]
fn actual_ast_records_resolved_call_edge() -> TestResult {
    let root = string_result(fixed_reference_root())?;
    let relative = "crates/katana-ui/src/shell_ui/shell_ui_frame/main_panels.rs";
    let source_path = root.join(relative);
    let source = fs::read_to_string(&source_path)?;
    let file = syn::parse_file(&source)?;
    let mut state = crate::source_closure::scan_state::ScanState::default();
    let mut discovered = Vec::new();
    crate::source_closure::ast_scan::SourceClosureVisitor::new(
        root,
        &source_path,
        relative,
        &mut state,
        &mut discovered,
    )
    .scan_file(&file);
    assert!(state.edges.iter().any(|edge| {
        edge.kind == "call"
            && edge.to_path.as_deref() == Some("crates/katana-ui/src/views/modals/terms.rs")
    }));
    Ok(())
}

#[test]
fn actual_reference_rejects_foreign_non_git_root_and_wrong_revision() -> TestResult {
    let fixture = fixture("actual-root-rejections")?;
    let verified = string_result(SourceClosureInputLoaderVerifier::load(&fixture.input_json))?;
    match ActualReferenceVerifier::bind(&verified, &fixture.root) {
        Ok(_) => return Err(test_error("non-git root must be rejected")),
        Err(error) => assert!(!error.is_empty(), "{error}"),
    }

    let kle_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .ok_or_else(|| test_error("KLE repository root missing"))?;
    match ActualReferenceVerifier::bind(&verified, kle_root) {
        Ok(_) => return Err(test_error("wrong revision must be rejected")),
        Err(error) => assert!(!error.is_empty(), "{error}"),
    }
    cleanup_dir(fixture.root)?;
    Ok(())
}

#[test]
fn actual_reference_rejects_modified_captured_source() -> TestResult {
    let fixture = fixture("actual-root-modified-source")?;
    mutate_input(&fixture, |value| {
        let evidence_path = option_result(
            value["root"]["evidence"]["katana_tree"][0]["path"]
                .as_str()
                .map(str::to_string),
            "tree evidence path missing",
        )?;
        let changed = b"captured bytes changed\n";
        fs::write(fixture.root.join("input").join(evidence_path), changed)?;
        value["root"]["evidence"]["katana_tree"][0]["sha256"] =
            serde_json::json!(sha256_hex(changed));
        update_tree_fingerprint(value)
    })?;
    let result = SourceClosureInputLoaderVerifier::load(&fixture.input_json);
    cleanup_dir(&fixture.root)?;
    match result {
        Ok(_) => Err(test_error("modified source capture must be rejected")),
        Err(error) => {
            assert!(!error.is_empty(), "{error}");
            Ok(())
        }
    }
}

#[test]
fn materializer_rejects_missing_discovered_module_capture() -> TestResult {
    let fixture = fixture("actual-root-missing-module")?;
    mutate_input(&fixture, |value| {
        let tree = option_result(
            value["root"]["evidence"]["katana_tree"].as_array_mut(),
            "tree evidence missing",
        )?;
        tree.remove(1);
        update_tree_fingerprint(value)
    })?;
    let result = SourceClosureInputLoaderVerifier::load(&fixture.input_json);
    cleanup_dir(&fixture.root)?;
    match result {
        Ok(_) => Err(test_error("missing discovered module capture must reject")),
        Err(error) => {
            assert!(!error.is_empty(), "{error}");
            Ok(())
        }
    }
}

fn update_tree_fingerprint(value: &mut serde_json::Value) -> TestResult {
    let mut records = option_result(
        value["root"]["evidence"]["katana_tree"].as_array(),
        "tree evidence missing",
    )?
    .iter()
    .map(|entry| {
        Ok(format!(
            "{}\0{}\n",
            option_result(entry["source_path"].as_str(), "source path missing")?,
            option_result(entry["sha256"].as_str(), "source SHA missing")?
        ))
    })
    .collect::<TestResult<Vec<_>>>()?;
    records.sort_unstable();
    value["root"]["katana_tree_fingerprint"] =
        serde_json::json!(sha256_hex(records.concat().as_bytes()));
    Ok(())
}
