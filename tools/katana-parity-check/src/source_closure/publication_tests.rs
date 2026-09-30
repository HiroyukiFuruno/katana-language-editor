use super::*;

#[test]
fn triple_publication_rolls_back_all_prior_artifacts_when_later_publish_fails()
-> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::temp_dir().join(format!(
        "kpc-pair-publication-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir_all(&root)?;
    let source_temp = root.join(".source-closure.json.tmp");
    let branch_temp = root.join(".branch-catalog.json.tmp");
    let action_temp = root.join(".action-origins.json.tmp");
    let leaf_temp = root.join(".leaf-manifest.json.tmp");
    let native_target_temp = root.join(".source-derived-native-target.json.tmp");
    let source_output = root.join("source-closure.json");
    let branch_output = root.join("branch-catalog.json");
    let action_output = root.join("action-origins.json");
    let leaf_output = root.join("leaf-manifest.json");
    let native_target_output = root.join("source-derived-native-target.json");
    let context_menu_temp = root.join(".context-menu-target-manifest.json.tmp");
    let context_menu_output = root.join("context-menu-target-manifest.json");
    fs::write(&source_temp, b"source")?;
    fs::write(&branch_temp, b"branch")?;

    let result = crate::source_closure::materializer::SourceClosureMaterializer::publish_prepared_artifacts_for_test(
        &crate::source_closure::materializer::PublicationPaths {
            source_temp: &source_temp,
            branch_temp: &branch_temp,
            action_temp: &action_temp,
            leaf_temp: &leaf_temp,
            native_target_temp: &native_target_temp,
            source_output: &source_output,
            branch_output: &branch_output,
            action_output: &action_output,
            leaf_output: &leaf_output,
            native_target_output: &native_target_output,
            context_menu_temp: &context_menu_temp,
            context_menu_output: &context_menu_output,
        },
    );

    let message = result.err().ok_or("branch publish must fail")?;
    assert!(message.contains("rolled back 2 published artifacts"));
    assert!(
        !source_output.exists(),
        "source final must not remain when branch final is absent"
    );
    assert!(!branch_output.exists());
    assert!(!action_output.exists());
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn materializer_refuses_existing_action_origins_artifact_before_any_publish()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = fixture("immutable-action-origins")?;
    let verified =
        SourceClosureInputLoaderVerifier::load(&fixture.input_json).map_err(test_error)?;
    let artifact_dir = fixture.root.join("artifacts");
    fs::create_dir_all(&artifact_dir)?;
    fs::write(artifact_dir.join("action-origins.json"), b"immutable")?;

    let result =
        SourceClosureMaterializer::materialize(&verified, &fixture.katana_root, &artifact_dir);

    let message = match result {
        Ok(_) => {
            return Err(test_error("immutable action origins must reject"));
        }
        Err(error) => error,
    };
    assert!(message.contains("refusing to overwrite immutable source-closure artifact"));
    assert!(!artifact_dir.join("source-closure.json").exists());
    assert!(!artifact_dir.join("branch-catalog.json").exists());
    cleanup_dir(fixture.root)?;
    Ok(())
}
