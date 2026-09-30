#[test]
fn binding_accepts_only_non_reusable_opaque_leases() {
    let source = include_str!("kuc_root_binding.rs");
    assert!(source.contains("EguiTextCommandSurfaceHostProjectionLease"));
    assert!(source.contains("EguiTextCommandSurfaceRootFactory::new().retain_with_lease"));
    assert!(source.contains(".synchronize_with_lease(lease)"));
    assert!(source.contains("forward_events_once(forwarder)"));
    assert!(!source.contains("EguiTextCommandSurfacePresentation"));
    assert!(!source.contains("EguiTextCommandSurfaceHostTarget"));
    assert!(!source.contains("KucRootEffectRouter"));
    assert!(!source.contains("KucRootEventBatchContext"));
    assert!(!source.contains("EguiTextCommandSurfaceRootEventTransport"));
    assert!(!source.contains("dispatch_once"));
    assert!(!source.contains("decode"));
    assert!(!source.contains("serialize"));
}

#[test]
fn stale_and_duplicate_paths_remain_typed_kuc_facade_errors() {
    let source = include_str!("kuc_root_binding.rs");
    assert!(source.contains("KucRootBindingError::Synchronize"));
    assert!(source.contains("KucRootBindingError::Forward"));
    assert!(source.contains("EguiTextCommandSurfaceRootFactoryError"));
    assert!(source.contains("EguiTextCommandSurfaceRootEventBatchForwardError"));
}

#[test]
fn storybook_artifact_path_keeps_writer_before_one_shot_forwarding() -> Result<(), String> {
    let source = include_str!("kuc_root_binding/artifact.rs");
    let storybook = source
        .split_once("fn write_current_artifact_and_forward_once")
        .map(|(_, body)| body)
        .ok_or_else(|| "storybook operation was not found".to_owned())?;
    let writer = storybook
        .find("write_artifact(&frame, output_dir, stage_id)")
        .ok_or_else(|| "KUC writer was not called".to_owned())?;
    let forward = storybook
        .find("forward_events_once(forwarder)")
        .ok_or_else(|| "one-shot forwarding was not called".to_owned())?;
    assert!(
        writer < forward,
        "writer must run before one-shot forwarding"
    );
    assert!(source.contains("OpaqueRootArtifactReceiptWriter::new()"));
    assert!(source.contains("map_err(KucRootBindingError::Artifact)"));
    assert!(source.contains("output_dir: &Path"));
    assert!(source.contains("stage_id: &str"));
    Ok(())
}

#[test]
fn storybook_artifact_path_preserves_stale_and_invalid_path_boundaries() {
    let source = include_str!("kuc_root_binding/artifact.rs");
    let binding = include_str!("kuc_root_binding.rs");
    assert!(source.contains("map_err(KucRootBindingError::Synchronize)"));
    assert!(source.contains("map_err(KucRootBindingError::Artifact)"));
    assert!(binding.contains("OpaqueRootArtifactReceiptError"));
    assert!(!source.contains("artifact_rgba"));
    assert!(!source.contains("png_bytes"));
    assert!(!source.contains("ArtifactCompositor"));
}
