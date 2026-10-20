#[path = "consumer_join_records_test_support.rs"]
mod support;

use super::super::super::super::artifact_model::{KatanaHostE2eState, StorybookArtifacts};
use super::super::super::super::execution_record_model::{ExecutionKind, ExecutionRecordArtifact};
use super::ConsumerJoinRecords;
use support::{artifact, artifact_with_receipt, cleanup, manifest, read_json};

#[test]
fn missing_profile_refuses_write() -> Result<(), String> {
    let (macos, artifact_root) = artifact("macos-latest", "consumer-stage-0000")?;
    let (manifest, manifest_root) = manifest()?;
    let staging = support::temp_dir("records-missing-profile")?;
    let mut records = ConsumerJoinRecords::new(manifest.leafs.clone());
    records.add_profile("macos-latest", &macos)?;
    let result = records.write(&staging, &manifest);
    cleanup(artifact_root)?;
    cleanup(manifest_root)?;
    cleanup(staging)?;
    assert!(matches!(result, Err(error) if error.contains("windows-latest")));
    Ok(())
}

#[test]
fn profile_mismatch_is_rejected() -> Result<(), String> {
    let (macos, artifact_root) = artifact("macos-latest", "consumer-stage-0000")?;
    let (manifest, manifest_root) = manifest()?;
    let mut records = ConsumerJoinRecords::new(manifest.leafs.clone());
    let result = records.add_profile("windows-latest", &macos);
    cleanup(artifact_root)?;
    cleanup(manifest_root)?;
    assert!(matches!(result, Err(error) if error.contains("differs from requested profile")));
    Ok(())
}

#[test]
fn duplicate_profile_is_rejected_without_mutation() -> Result<(), String> {
    let (macos, macos_root) = artifact("macos-latest", "consumer-stage-0000")?;
    let (windows, windows_root) = artifact("windows-latest", "consumer-stage-0000")?;
    let (manifest, manifest_root) = manifest()?;
    let staging = support::temp_dir("records-duplicate-profile")?;
    let mut records = ConsumerJoinRecords::new(manifest.leafs.clone());
    records.add_profile("macos-latest", &macos)?;
    let duplicate = records.add_profile("macos-latest", &macos);
    assert!(duplicate.is_err());
    records.add_profile("windows-latest", &windows)?;
    records.write(&staging, &manifest)?;
    cleanup(macos_root)?;
    cleanup(windows_root)?;
    cleanup(manifest_root)?;
    cleanup(staging)
}

#[test]
fn reused_receipt_is_rejected_across_profiles_without_mutation() -> Result<(), String> {
    let (macos, macos_root) = artifact("macos-latest", "consumer-stage-0000")?;
    let (reused, reused_root) =
        artifact_with_receipt("windows-latest", "consumer-stage-0000", 'e')?;
    let (windows, windows_root) = artifact("windows-latest", "consumer-stage-0000")?;
    let (manifest, manifest_root) = manifest()?;
    let mut records = ConsumerJoinRecords::new(manifest.leafs.clone());
    records.add_profile("macos-latest", &macos)?;
    let result = records.add_profile("windows-latest", &reused);
    assert!(matches!(
        result,
        Err(error) if error.contains("receipt fingerprint is reused")
    ));
    records.add_profile("windows-latest", &windows)?;
    assert_eq!(records.receipts.len(), 2);
    cleanup(macos_root)?;
    cleanup(reused_root)?;
    cleanup(windows_root)?;
    cleanup(manifest_root)
}

#[test]
fn same_leaf_stage_mismatch_is_rejected_during_write() -> Result<(), String> {
    let (macos, macos_root) = artifact("macos-latest", "consumer-stage-0000")?;
    let (manifest, manifest_root) = manifest()?;
    let mut records = ConsumerJoinRecords::new(manifest.leafs.clone());
    records.add_profile("macos-latest", &macos)?;
    records.stages.insert(
        ("leaf-a".into(), "windows-latest".into()),
        super::ConsumerStage {
            stage_id: "consumer-stage-0001".into(),
            interaction: "text_input".into(),
            media_sha256: "a".repeat(support::HASH_LEN),
            root_record_hash: "b".repeat(support::HASH_LEN),
        },
    );
    let staging = support::temp_dir("records-stage-mismatch")?;
    let result = records.write(&staging, &manifest);
    cleanup(macos_root)?;
    cleanup(manifest_root)?;
    cleanup(staging)?;
    assert!(matches!(result, Err(error) if error.contains("stage differs")));
    Ok(())
}

#[test]
fn write_generates_consumer_records_and_keeps_downstream_required() -> Result<(), String> {
    let (macos, macos_root) = artifact("macos-latest", "consumer-stage-0000")?;
    let (windows, windows_root) = artifact("windows-latest", "consumer-stage-0000")?;
    let (manifest, manifest_root) = manifest()?;
    let staging = support::temp_dir("records-output")?;
    let mut records = ConsumerJoinRecords::new(manifest.leafs.clone());
    records.add_profile("macos-latest", &macos)?;
    records.add_profile("windows-latest", &windows)?;
    records.write(&staging, &manifest)?;
    let output: super::super::super::super::artifact_model::LeafManifestArtifact =
        read_json(&staging.join("leaf-manifest.json"))?;
    let leaf = output.leafs.first().ok_or("output leaf is missing")?;
    assert_eq!(leaf.status, "passing");
    assert_eq!(leaf.execution_id, None);
    assert_eq!(leaf.host_e2e.state, KatanaHostE2eState::DownstreamRequired);
    let executions: ExecutionRecordArtifact = read_json(&staging.join("execution-record.json"))?;
    assert_eq!(executions.executions.len(), 2);
    assert!(executions.executions.iter().all(|execution| {
        matches!(execution.execution_kind, ExecutionKind::KucConsumerArtifact)
            && execution.status == "passing"
    }));
    let stories: StorybookArtifacts = read_json(&staging.join("storybook-artifacts.json"))?;
    assert_eq!(stories.artifacts.len(), 2);
    assert_eq!(
        stories.artifacts[0].story_id,
        "consumer:macos-latest:consumer-stage-0000"
    );
    cleanup(macos_root)?;
    cleanup(windows_root)?;
    cleanup(manifest_root)?;
    cleanup(staging)
}
