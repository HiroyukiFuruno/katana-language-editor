use std::fs;
use std::path::PathBuf;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn storybook_contract_check_keeps_kle_fixture_and_text_geometry_out_of_host_types() -> TestResult {
    let source =
        fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/host_types.rs"))?;
    assert_storybook_provider_uses_kuc_session(&source);
    assert_storybook_provider_has_no_kle_owned_surface(&source);
    Ok(())
}

fn assert_storybook_provider_uses_kuc_session(source: &str) {
    assert!(
        source.contains("FullTextCommandSurfaceScenarioSession"),
        "the Storybook provider must retain the KUC-owned scenario session"
    );
}

fn assert_storybook_provider_has_no_kle_owned_surface(source: &str) {
    for forbidden in [
        "TextSurface",
        "TextArea",
        "TextSurfaceViewport",
        "EguiTextCommandSurfacePresentation",
        "TextCommandSurfaceStyle",
        "# Generic text surface",
        "日本語と ASCII",
        "exact ⭐️ VS16",
        "selection_start",
        "selection_end",
        "pos2(",
        "KucRootEventBatchContext",
        "KucOpaqueHostEffectBatch",
        "StorybookGenericEffectReceipt",
        "StorybookOpaqueReceiptHost",
    ] {
        assert!(
            !source.contains(forbidden),
            "host_types.rs contains `{forbidden}`"
        );
    }
}
