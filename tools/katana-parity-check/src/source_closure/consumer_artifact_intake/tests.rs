use super::ConsumerArtifactIntake;
use super::tests_support::{fixture, plain_hash, read_evidence, write_evidence};
use serde_json::{Value, json};
use std::fs;

fn rejected(mut edit: impl FnMut(&mut Value)) -> Result<(), String> {
    let (plan, root) = fixture()?;
    let mut evidence = read_evidence(&root)?;
    edit(&mut evidence);
    write_evidence(&root, &evidence)?;
    let result = ConsumerArtifactIntake::validate_profile_artifact(&plan, &root);
    fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    if result.is_ok() {
        return Err("invalid artifact was accepted".into());
    }
    Ok(())
}

#[test]
fn accepts_two_stage_contract() -> Result<(), String> {
    let (plan, root) = fixture()?;
    let result = ConsumerArtifactIntake::validate_profile_artifact(&plan, &root);
    fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    result
}

#[test]
fn rejects_missing_duplicate_reordered_unknown_and_drifted_stages() -> Result<(), String> {
    rejected(|e| {
        if let Some(stages) = e["stages"].as_array_mut() {
            stages.pop();
        }
    })?;
    rejected(|e| {
        let duplicate = e["stages"][0].clone();
        if let Some(stages) = e["stages"].as_array_mut() {
            stages.push(duplicate);
        }
    })?;
    rejected(|e| {
        if let Some(stages) = e["stages"].as_array_mut() {
            stages.swap(0, 1);
        }
    })?;
    rejected(|e| e["stages"][0]["leaf_id"] = json!("unknown"))?;
    rejected(|e| e["stages"][0]["interaction"] = json!("search"))?;
    Ok(())
}

#[test]
fn rejects_provenance_media_and_revision_drift() -> Result<(), String> {
    for field in [
        "run_id",
        "source_closure_sha256",
        "profile_fingerprint",
        "profile_id",
    ] {
        rejected(|e| e[field] = "foreign".into())?;
    }
    rejected(|e| e["stages"][0]["media_sha256"] = json!(plain_hash(b"foreign")))?;
    rejected(|e| e["stages"][0]["root_revision"] = json!(0))?;
    rejected(|e| e["stages"][0]["media_path"] = json!("../outside.png"))?;
    rejected(|e| e["stages"][0]["media_path"] = json!("C:\\outside.png"))?;
    let (plan, root) = fixture()?;
    fs::write(root.join("consumer-stage-0000.png"), b"modified")
        .map_err(|error| error.to_string())?;
    let error = ConsumerArtifactIntake::validate_profile_artifact(&plan, &root)
        .err()
        .ok_or("modified media was accepted")?;
    fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    assert!(error.contains("media SHA-256 mismatch"));
    Ok(())
}

#[test]
fn rejects_receipt_reuse_and_unicode_rebinding() -> Result<(), String> {
    let (plan, root) = fixture()?;
    let first_receipt = fs::read_to_string(root.join("consumer-stage-0000.consumer-artifact.json"))
        .map_err(|error| error.to_string())?;
    let first_manifest: Value =
        serde_json::from_str(&first_receipt).map_err(|error| error.to_string())?;
    let second_path = root.join("consumer-stage-0001.consumer-artifact.json");
    let mut second_manifest: Value =
        serde_json::from_slice(&fs::read(&second_path).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
    second_manifest["forwarding_receipt_fingerprint"] =
        first_manifest["forwarding_receipt_fingerprint"].clone();
    fs::write(
        &second_path,
        serde_json::to_vec(&second_manifest).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let receipt_error = ConsumerArtifactIntake::validate_profile_artifact(&plan, &root)
        .err()
        .ok_or_else(|| "receipt reuse was accepted".to_string())?;
    if !receipt_error.contains("duplicate receipt fingerprint") {
        return Err(format!("unexpected receipt reuse error: {receipt_error}"));
    }
    fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    let (plan, root) = fixture()?;
    let mut evidence = read_evidence(&root)?;
    let rebound = plain_hash(b"rebound");
    evidence["stages"][0]["unicode_evidence_hash"] = json!(rebound);
    write_evidence(&root, &evidence)?;
    let manifest_path = root.join("consumer-stage-0000.consumer-artifact.json");
    let mut manifest: Value =
        serde_json::from_slice(&fs::read(&manifest_path).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
    manifest["unicode_evidence_hash"] = json!(rebound);
    fs::write(
        &manifest_path,
        serde_json::to_vec(&manifest).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let rebind_error = ConsumerArtifactIntake::validate_profile_artifact(&plan, &root)
        .err()
        .ok_or_else(|| "Unicode metadata rebind was accepted".to_string())?;
    if !rebind_error.contains("binding mismatch") {
        return Err(format!("unexpected Unicode rebind error: {rebind_error}"));
    }
    fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    let (plan, root) = fixture()?;
    fs::write(
        root.join("consumer-stage-0000.unicode-evidence.json"),
        b"changed",
    )
    .map_err(|error| error.to_string())?;
    if ConsumerArtifactIntake::validate_profile_artifact(&plan, &root).is_ok() {
        return Err("Unicode payload mutation was accepted".into());
    }
    fs::remove_dir_all(root).map_err(|error| error.to_string())
}

#[cfg(unix)]
#[test]
fn rejects_symlinked_media() -> Result<(), String> {
    let (plan, root) = fixture()?;
    let target = root.join("consumer-stage-0000.png");
    let real = root.join("real.png");
    fs::rename(&target, root.join("real.png")).map_err(|error| error.to_string())?;
    std::os::unix::fs::symlink(&real, &target).map_err(|error| error.to_string())?;
    if ConsumerArtifactIntake::validate_profile_artifact(&plan, &root).is_ok() {
        return Err("symlinked media was accepted".into());
    }
    fs::remove_dir_all(root).map_err(|error| error.to_string())
}

#[cfg(windows)]
#[test]
fn rejects_symlinked_media() -> Result<(), String> {
    let (plan, root) = fixture()?;
    let target = root.join("consumer-stage-0000.png");
    let real = root.join("real.png");
    fs::rename(&target, &real).map_err(|error| error.to_string())?;
    std::os::windows::fs::symlink_file(&real, &target).map_err(|error| error.to_string())?;
    if ConsumerArtifactIntake::validate_profile_artifact(&plan, &root).is_ok() {
        return Err("symlinked media was accepted".into());
    }
    fs::remove_dir_all(root).map_err(|error| error.to_string())
}
