use crate::release_gate::ReleaseGateAudit;

pub(crate) struct GenericInteractionAnnotationCatalogAudit;

impl GenericInteractionAnnotationCatalogAudit {
    pub(crate) fn validate(assemble: &[&str]) -> Result<(), String> {
        validate_catalog_generation(assemble)?;
        validate_catalog_upload(assemble)?;
        validate_catalog_ordering(assemble)
    }
}

fn validate_catalog_generation(assemble: &[&str]) -> Result<(), String> {
    let catalog =
        ReleaseGateAudit::step_section(assemble, "Emit generic interaction annotation catalog")?;
    let command = catalog
        .iter()
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join(" ");
    for expected in [
        "source-closure emit-generic-annotation-catalog",
        "--input artifacts/v0-1-0/source-closure-input/4f6a6287c650a38633c7baeb544a92e739c68567/artifacts/leaf-manifest.json",
        "--source-closure artifacts/v0-1-0/source-closure-input/4f6a6287c650a38633c7baeb544a92e739c68567/artifacts/source-closure.json",
        "--branches artifacts/v0-1-0/source-closure-input/4f6a6287c650a38633c7baeb544a92e739c68567/artifacts/branch-catalog.json",
        "--output target/source-closure/${{ env.SOURCE_CLOSURE_RUN_ID }}/generic-interaction-annotation-catalog.json",
    ] {
        if !command.contains(expected) {
            return Err(format!(
                "source-closure generic interaction annotation catalog step is missing `{expected}`"
            ));
        }
    }
    Ok(())
}

fn validate_catalog_upload(assemble: &[&str]) -> Result<(), String> {
    let upload =
        ReleaseGateAudit::step_section(assemble, "Upload generic interaction annotation catalog")?;
    for expected in [
        "uses: actions/upload-artifact@v4",
        "name: source-closure-generic-interaction-annotations-${{ github.run_id }}",
        "path: katana-language-editor/target/source-closure/${{ env.SOURCE_CLOSURE_RUN_ID }}/generic-interaction-annotation-catalog.json",
        "if-no-files-found: error",
    ] {
        if !upload.iter().any(|line| line.trim() == expected) {
            return Err(format!(
                "source-closure generic interaction annotation upload is missing `{expected}`"
            ));
        }
    }
    Ok(())
}

fn validate_catalog_ordering(assemble: &[&str]) -> Result<(), String> {
    let catalog_index = step_index(assemble, "Emit generic interaction annotation catalog")?;
    let upload_index = step_index(assemble, "Upload generic interaction annotation catalog")?;
    let validation_index = step_index(assemble, "Validate KLE release parity")?;
    if catalog_index < upload_index && upload_index < validation_index {
        Ok(())
    } else {
        Err(
            "source-closure must upload the immutable annotation catalog before release parity validation"
                .to_string(),
        )
    }
}

fn step_index(lines: &[&str], step_name: &str) -> Result<usize, String> {
    let expected = format!("- name: {step_name}");
    lines
        .iter()
        .position(|line| line.trim() == expected)
        .ok_or_else(|| format!("source-closure `{step_name}` step is missing"))
}
