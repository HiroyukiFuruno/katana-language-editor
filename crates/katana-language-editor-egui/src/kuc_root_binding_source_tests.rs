#[test]
fn binding_retains_only_the_kuc_host_root() -> Result<(), String> {
    let source = include_str!("kuc_root_binding.rs");
    let fields = source
        .split_once("pub struct KucRootBinding {")
        .and_then(|(_, rest)| rest.split_once("}\n\n#[derive"))
        .map(|(fields, _)| fields)
        .ok_or("binding definition was not found")?;
    assert!(fields.contains("root: EguiTextCommandSurfaceHostRoot"));
    assert!(!fields.contains("pub "));
    Ok(())
}

#[test]
fn receipt_contains_only_closed_record_and_forwarding_values() -> Result<(), String> {
    let source = include_str!("kuc_root_binding.rs");
    let fields = receipt_fields(source)?;
    assert_receipt_fields(fields);
    assert_receipt_does_not_leak(fields);
    Ok(())
}

#[cfg(test)]
fn receipt_fields(source: &str) -> Result<&str, String> {
    source
        .split_once("pub struct KucRootBindingReceipt {")
        .and_then(|(_, rest)| rest.split_once("}\n\n#[path = \"kuc_root_binding_receipt.rs\"]"))
        .map(|(fields, _)| fields)
        .ok_or_else(|| "receipt definition was not found".to_owned())
}

#[cfg(test)]
fn assert_receipt_fields(fields: &str) {
    for field in [
        "root_identity: String",
        "presentation_revision: u64",
        "state_revision: u64",
        "dimensions: EguiTextCommandSurfaceHostRootRecordDimensions",
        "paint_plan_hash: String",
        "record_hash: String",
        "accessibility_snapshot_hash: String",
        "correlation_fingerprint: String",
        "event_batch_fingerprint: String",
        "event_cardinality: usize",
        "consumed_once: bool",
    ] {
        assert!(fields.contains(field), "receipt lost `{field}`");
    }
}

#[cfg(test)]
fn assert_receipt_does_not_leak(fields: &str) {
    for forbidden in [
        "payload",
        "target",
        "child",
        "artifact",
        "rgba_pixels",
        "geometry",
        "accessibility_node",
        "semantic_action",
        "SanitizedDocumentRoot",
        "EguiTextCommandSurfaceRootOutput",
        "EguiTextCommandSurfacePresentation",
        "TextCommandSurfaceStyle",
    ] {
        assert!(!fields.contains(forbidden), "receipt leaked `{forbidden}`");
    }
}

#[test]
fn binding_has_no_old_sanitized_surface_or_local_projection_builder() {
    let source = include_str!("kuc_root_binding.rs");
    for forbidden in [
        "EguiTextCommandSurfacePresentation",
        "EguiTextCommandSurfaceHostTarget",
        "SanitizedDocumentRoot",
        "SanitizedDocumentRootInput",
        "SanitizedDocumentRootEventForwarder",
        "EguiTextCommandSurfaceHostProjectionEncoder",
        "EguiTextCommandSurfacePresentation::",
        "TextCommandSurfaceStyle",
        "EguiTextCommandSurfaceChild",
        "legacy_artifact_aggregate",
    ] {
        assert!(
            !source.contains(forbidden),
            "binding contains `{forbidden}`"
        );
    }
}
