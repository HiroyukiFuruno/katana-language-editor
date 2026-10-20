use super::*;
use crate::source_closure::execution_coverage;
use crate::source_closure::model::{ExecutionKind, ExecutionRecord};
use std::collections::BTreeSet;

fn fixture() -> Result<(ExecutionRecordArtifact, LeafManifestArtifact), String> {
    let directory = FixtureBuilder::root()?;
    let paths = complete_artifacts(directory.path())?;
    Ok((
        read_artifact(&paths.execution_record)?,
        read_artifact(&paths.leaf_manifest)?,
    ))
}

#[test]
fn old_records_remain_host_claims_and_unknown_kinds_are_rejected() -> Result<(), String> {
    let mut json = serde_json::json!({
        "execution_id":"execution", "execution_profile_id":"macos-latest",
        "leaf_id":"leaf", "action_route_signature":"route", "status":"passing"
    });
    let legacy: ExecutionRecord =
        serde_json::from_value(json.clone()).map_err(|error| error.to_string())?;
    assert!(matches!(legacy.execution_kind, ExecutionKind::KatanaHost));
    json["execution_kind"] = "unknown".into();
    assert!(serde_json::from_value::<ExecutionRecord>(json).is_err());
    Ok(())
}

#[test]
fn consumer_executions_never_satisfy_host_coverage() -> Result<(), String> {
    let (mut executions, manifest) = fixture()?;
    let mut errors = Vec::new();
    execution_coverage::validate_kle_boundary(&executions, &manifest, &mut errors);
    assert!(errors.iter().any(|error| error.contains("only KUC")));
    for execution in &mut executions.executions {
        execution.execution_kind = ExecutionKind::KucConsumerArtifact;
    }
    errors.clear();
    execution_coverage::validate_kle_boundary(&executions, &manifest, &mut errors);
    assert!(errors.is_empty(), "{errors:?}");
    let leaf_ids = manifest
        .leafs
        .iter()
        .map(|leaf| leaf.leaf_id.clone())
        .collect::<BTreeSet<_>>();
    execution_coverage::validate_full(&executions, &manifest, &leaf_ids, &mut errors);
    assert!(
        errors
            .iter()
            .any(|error| error.contains("missing required leaf execution"))
    );
    errors.clear();
    execution_coverage::validate_rc_boundary(&executions, &mut errors);
    assert!(!errors.is_empty());
    Ok(())
}

#[test]
fn consumer_coverage_rejects_missing_and_duplicate_profiles() -> Result<(), String> {
    let (mut executions, manifest) = fixture()?;
    for execution in &mut executions.executions {
        execution.execution_kind = ExecutionKind::KucConsumerArtifact;
    }
    let removed = executions
        .executions
        .pop()
        .ok_or("fixture has no execution")?;
    let mut errors = Vec::new();
    execution_coverage::validate_kle_boundary(&executions, &manifest, &mut errors);
    assert!(errors.iter().any(|error| error.contains("exactly once")));
    executions.executions.push(removed.clone());
    executions.executions.push(removed);
    errors.clear();
    execution_coverage::validate_kle_boundary(&executions, &manifest, &mut errors);
    assert!(errors.iter().any(|error| error.contains("exactly once")));
    Ok(())
}
