use super::super::artifact_validator::{ArtifactValidationMode, SourceClosureArtifactValidator};
use super::super::load::read_artifact;
use super::super::model::{
    ActionOriginsArtifact, BranchCatalogArtifact, ExecutionRecordArtifact, KatanaHostE2eState,
    LeafManifestArtifact, SourceClosureArtifact, StorybookArtifacts,
};
use super::validator_test_support::{complete_artifacts, write_file};
use crate::capability_manifest::capability_manifest_test_fixtures::FixtureBuilder;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[path = "validator_tests/validator_artifact_boundary.rs"]
mod validator_artifact_boundary;
#[path = "validator_tests/validator_external.rs"]
mod validator_external;
#[path = "validator_tests/validator_mode.rs"]
mod validator_mode;

fn validation_error_message<T>(
    result: Result<T, String>,
    message: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    match result {
        Ok(_) => Err(Box::new(std::io::Error::other(message.to_string()))),
        Err(error) => Ok(error),
    }
}

fn unique_temp_dir(prefix: &str) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    Ok(std::env::temp_dir().join(format!("{prefix}-{suffix}")))
}

#[test]
fn full_rejects_explicit_downstream_host_boundary() -> TestResult {
    let dir = unique_temp_dir("katana-parity-source-closure-full-downstream")?;
    let paths = complete_artifacts(&dir)?;
    let mut leaf_root: LeafManifestArtifact = read_artifact(&paths.leaf_manifest)?;
    leaf_root.leafs[0].host_e2e.state = KatanaHostE2eState::DownstreamRequired;
    leaf_root.leafs[0].execution_id = None;
    write_file(&paths.leaf_manifest, &leaf_root)?;
    let mut execution_root: ExecutionRecordArtifact = read_artifact(&paths.execution_record)?;
    execution_root.executions.clear();
    write_file(&paths.execution_record, &execution_root)?;

    let result =
        SourceClosureArtifactValidator::validate_for_mode(&paths, ArtifactValidationMode::Full);
    let message = validation_error_message(result, "expected full host E2E rejection")?;
    assert!(message.contains("requires a passing KatanA host E2E execution id"));
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn rc_rejects_premature_host_execution_claim() -> TestResult {
    let dir = unique_temp_dir("katana-parity-source-closure-rc-host-claim")?;
    let paths = complete_artifacts(&dir)?;

    let result =
        SourceClosureArtifactValidator::validate_for_mode(&paths, ArtifactValidationMode::Rc);
    let message = validation_error_message(result, "expected RC host execution rejection")?;
    assert!(message.contains("must declare downstream_required"));
    assert!(message.contains("must not claim KatanA host execution"));
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

#[cfg(unix)]
#[test]
fn kle_release_rejects_symlinked_candidate_artifact() -> TestResult {
    use std::os::unix::fs::symlink;

    let fixture = FixtureBuilder::root()?;
    let paths = complete_artifacts(fixture.path())?;
    let outside = fixture.path().join("outside-kle-release-evidence.json");
    std::fs::write(&outside, b"{}")?;
    symlink(&outside, &paths.kle_release_evidence)?;

    let result = SourceClosureArtifactValidator::validate_for_mode(
        &paths,
        ArtifactValidationMode::KleRelease,
    );
    let message = validation_error_message(result, "expected symlink candidate rejection")?;
    assert!(message.contains("KLE release evidence must not be a symlink"));
    Ok(())
}

#[test]
fn rejects_profile_mismatch() -> TestResult {
    let dir = unique_temp_dir("katana-parity-source-closure-profile-mismatch")?;
    let paths = complete_artifacts(&dir)?;
    let mut action_root: ActionOriginsArtifact = read_artifact(&paths.action_origins)?;
    action_root.actions[0].active_profile_ids = vec!["ubuntu-latest".to_string()];
    write_file(&paths.action_origins, &action_root)?;

    let result =
        SourceClosureArtifactValidator::validate_for_mode(&paths, ArtifactValidationMode::Full);
    let message = validation_error_message(result, "expected profile mismatch rejection")?;
    assert!(message.contains("invalid/missing"));
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn rejects_static_leaf_claims() -> TestResult {
    let dir = unique_temp_dir("katana-parity-source-closure-static-leaf")?;
    let paths = complete_artifacts(&dir)?;
    let mut leaf_root: LeafManifestArtifact = read_artifact(&paths.leaf_manifest)?;
    leaf_root.root.static_leaf_count = Some(1);
    write_file(&paths.leaf_manifest, &leaf_root)?;

    let result =
        SourceClosureArtifactValidator::validate_for_mode(&paths, ArtifactValidationMode::Full);
    let message = validation_error_message(result, "expected static claim rejection")?;
    assert!(message.contains("static leaf count"));
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn rejects_candidate_leaf_status() -> TestResult {
    let dir = unique_temp_dir("katana-parity-source-closure-leaf-status-candidate")?;
    let paths = complete_artifacts(&dir)?;
    let mut leaf_root: LeafManifestArtifact = read_artifact(&paths.leaf_manifest)?;
    leaf_root.leafs[0].status = "candidate".to_string();
    write_file(&paths.leaf_manifest, &leaf_root)?;

    let result =
        SourceClosureArtifactValidator::validate_for_mode(&paths, ArtifactValidationMode::Full);
    let message = validation_error_message(result, "expected candidate leaf status rejection")?;
    assert!(message.contains("invalid status"));
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn rejects_pending_leaf_status() -> TestResult {
    let dir = unique_temp_dir("katana-parity-source-closure-leaf-status-pending")?;
    let paths = complete_artifacts(&dir)?;
    let mut leaf_root: LeafManifestArtifact = read_artifact(&paths.leaf_manifest)?;
    leaf_root.leafs[0].status = "pending".to_string();
    write_file(&paths.leaf_manifest, &leaf_root)?;

    let result =
        SourceClosureArtifactValidator::validate_for_mode(&paths, ArtifactValidationMode::Full);
    let message = validation_error_message(result, "expected pending leaf status rejection")?;
    assert!(message.contains("invalid status"));
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn rejects_not_executed_execution_status() -> TestResult {
    let dir = unique_temp_dir("katana-parity-source-closure-execution-status-not-executed")?;
    let paths = complete_artifacts(&dir)?;
    let mut execution_root: ExecutionRecordArtifact = read_artifact(&paths.execution_record)?;
    execution_root.executions[0].status = "not_executed".to_string();
    write_file(&paths.execution_record, &execution_root)?;

    let result =
        SourceClosureArtifactValidator::validate_for_mode(&paths, ArtifactValidationMode::Full);
    let message =
        validation_error_message(result, "expected not_executed execution status rejection")?;
    assert!(
        message.contains(
            "execution-record execution exec-format-bold has placeholder or invalid fields"
        )
    );
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn rejects_missing_execution_and_storybook_coverage_for_declared_leaf() -> TestResult {
    let dir = unique_temp_dir("katana-parity-source-closure-missing-leaf-coverage")?;
    let paths = complete_artifacts(&dir)?;
    let mut leaf_root: LeafManifestArtifact = read_artifact(&paths.leaf_manifest)?;
    let mut uncovered = leaf_root.leafs[0].clone();
    uncovered.leaf_id = "menu.editor.format_italic".to_string();
    uncovered.execution_id = Some("exec-format-italic".to_string());
    uncovered.storybook_stage_id = "story-format-italic".to_string();
    leaf_root.leafs.push(uncovered);
    write_file(&paths.leaf_manifest, &leaf_root)?;

    let result =
        SourceClosureArtifactValidator::validate_for_mode(&paths, ArtifactValidationMode::Full);
    let message = validation_error_message(result, "expected missing leaf coverage rejection")?;
    assert!(message.contains("execution-record is missing required leaf execution"));
    assert!(message.contains("storybook-artifacts is missing required leaf artifact"));
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn rejects_storybook_stage_mismatch_for_declared_leaf() -> TestResult {
    let dir = unique_temp_dir("katana-parity-source-closure-storybook-stage-mismatch")?;
    let paths = complete_artifacts(&dir)?;
    let mut leaf_root: LeafManifestArtifact = read_artifact(&paths.leaf_manifest)?;
    leaf_root.leafs[0].storybook_stage_id = "story-format-italic".to_string();
    write_file(&paths.leaf_manifest, &leaf_root)?;

    let result =
        SourceClosureArtifactValidator::validate_for_mode(&paths, ArtifactValidationMode::Full);
    let message = validation_error_message(result, "expected storybook stage mismatch rejection")?;
    assert!(message.contains("storybook-artifacts is missing declared stage"));
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn rejects_missing_execution_profile_for_declared_leaf() -> TestResult {
    let dir = unique_temp_dir("katana-parity-source-closure-missing-execution-profile")?;
    let paths = complete_artifacts(&dir)?;
    let mut execution_root: ExecutionRecordArtifact = read_artifact(&paths.execution_record)?;
    execution_root
        .executions
        .retain(|record| record.execution_profile_id == "ubuntu-latest");
    write_file(&paths.execution_record, &execution_root)?;

    let result =
        SourceClosureArtifactValidator::validate_for_mode(&paths, ArtifactValidationMode::Full);
    let message = validation_error_message(result, "expected missing execution profile rejection")?;
    assert!(message.contains("execution-record is missing required profile"));
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn rejects_missing_storybook_profile_for_declared_leaf() -> TestResult {
    let dir = unique_temp_dir("katana-parity-source-closure-missing-storybook-profile")?;
    let paths = complete_artifacts(&dir)?;
    let mut story_root: StorybookArtifacts = read_artifact(&paths.storybook_artifacts)?;
    story_root
        .artifacts
        .retain(|record| record.profile_id == "ubuntu-latest");
    write_file(&paths.storybook_artifacts, &story_root)?;

    let result =
        SourceClosureArtifactValidator::validate_for_mode(&paths, ArtifactValidationMode::Full);
    let message = validation_error_message(result, "expected missing storybook profile rejection")?;
    assert!(message.contains("storybook-artifacts is missing required profile"));
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn rejects_missing_storybook_media_file() -> TestResult {
    let dir = unique_temp_dir("katana-parity-source-closure-missing-storybook-media")?;
    let paths = complete_artifacts(&dir)?;
    let story_root: StorybookArtifacts = read_artifact(&paths.storybook_artifacts)?;
    std::fs::remove_file(dir.join(&story_root.artifacts[0].media_path))?;

    let result = SourceClosureArtifactValidator::validate(&paths);
    let message = validation_error_message(result, "expected missing storybook media rejection")?;
    assert!(message.contains("media is missing"));
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn rejects_storybook_media_hash_mismatch() -> TestResult {
    let dir = unique_temp_dir("katana-parity-source-closure-storybook-media-hash")?;
    let paths = complete_artifacts(&dir)?;
    let mut story_root: StorybookArtifacts = read_artifact(&paths.storybook_artifacts)?;
    story_root.artifacts[0].media_sha256 = super::super::fingerprint::sha256_hex(b"tampered hash");
    write_file(&paths.storybook_artifacts, &story_root)?;

    let result = SourceClosureArtifactValidator::validate(&paths);
    let message = validation_error_message(result, "expected storybook media hash rejection")?;
    assert!(message.contains("media hash does not match"));
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn rejects_non_png_or_unnumbered_storybook_media() -> TestResult {
    let dir = unique_temp_dir("katana-parity-source-closure-invalid-storybook-media")?;
    let paths = complete_artifacts(&dir)?;
    let mut story_root: StorybookArtifacts = read_artifact(&paths.storybook_artifacts)?;
    let artifact = &mut story_root.artifacts[0];
    artifact.media_path = "media/story-format-bold-1.png".to_string();
    let bytes = b"\x89PNG\r\n\x1a\n";
    artifact.media_sha256 = super::super::fingerprint::sha256_hex(bytes);
    std::fs::write(dir.join(&artifact.media_path), bytes)?;
    write_file(&paths.storybook_artifacts, &story_root)?;

    let result = SourceClosureArtifactValidator::validate(&paths);
    let message = validation_error_message(result, "expected invalid storybook media rejection")?;
    assert!(message.contains("must be a numbered PNG stage"));
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}
