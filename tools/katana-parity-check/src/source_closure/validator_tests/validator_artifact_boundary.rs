use super::{
    ArtifactValidationMode, SourceClosureArtifactValidator, TestResult, complete_artifacts,
    unique_temp_dir, validation_error_message,
};

#[test]
fn rejects_missing_artifact() -> TestResult {
    let dir = unique_temp_dir("katana-parity-source-closure-missing")?;
    let paths = complete_artifacts(&dir)?;
    std::fs::remove_file(&paths.branch_catalog)?;
    let result =
        SourceClosureArtifactValidator::validate_for_mode(&paths, ArtifactValidationMode::Full);
    let message = validation_error_message(result, "expected missing artifact rejection")?;
    assert!(message.contains("branch-catalog"));
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}
