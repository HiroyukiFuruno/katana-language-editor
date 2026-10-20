use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::super::operational_input::{EvidenceRef, ProfileProbeInput, SourceClosureInput};
use super::super::operational_loader::SourceClosureInputLoaderVerifier;
use super::super::operational_publication::publish;
use super::super::operational_staging::{validate_validation_receipt, write_validation_receipt};
use super::super::operational_staging_layout::validate_profile_layout;
use super::operational_fixture::fixture;

const CAPTURE_ID_PARTS: usize = 3;

struct StagingFixture {
    root: PathBuf,
    staging: PathBuf,
    input: PathBuf,
}

fn staging_fixture(name: &str) -> Result<StagingFixture, Box<dyn std::error::Error>> {
    let fixture = fixture(name)?;
    let staging = fixture.root.join("run-1");
    fs::create_dir_all(&staging)?;
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&fixture.input_json)?)?;
    rewrite_value(&mut value, "run-1");
    rewrite_raw_paths(&mut value);
    let mut input_value: SourceClosureInput = serde_json::from_value(value)?;
    input_value.root.katana_external_ui_fingerprint =
        EvidenceRef::set_fingerprint(&input_value.root.evidence.katana_external_ui);
    input_value.root.kle_tree_fingerprint =
        EvidenceRef::set_fingerprint(&input_value.root.evidence.kle_tree);
    input_value.root.kuc_tree_fingerprint =
        EvidenceRef::set_fingerprint(&input_value.root.evidence.kuc_tree);
    input_value.root.generator_fingerprint = EvidenceRef::set_fingerprint(&[
        input_value.root.evidence.generator_binary.clone(),
        input_value.root.evidence.generator_schema.clone(),
    ]);
    input_value.root.release_profile_matrix_fingerprint = input_value.matrix_fingerprint();
    for profile in &mut input_value.profile_probes {
        profile.fingerprint = profile.identity_fingerprint();
    }
    input_value.root.release_profile_matrix_fingerprint = input_value.matrix_fingerprint();
    let fixture_parent = fixture
        .input_json
        .parent()
        .ok_or_else(|| io::Error::other("fixture input path has no parent"))?;
    fs::rename(fixture_parent.join("profiles"), staging.join("profiles"))?;
    fs::rename(
        fixture_parent.join("raw/provenance"),
        staging.join("provenance"),
    )?;
    for profile in &input_value.profile_probes {
        let profile_dir = staging.join("profiles").join(&profile.id);
        super::super::operational_output::write_canonical_json(
            &profile_dir.join("probe.json"),
            profile,
        )?;
        let mut entries = profile_evidence(profile)
            .into_iter()
            .map(|evidence| super::super::operational_output::ChecksumEntry {
                path: evidence.path,
                sha256: evidence.sha256,
            })
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| left.path.cmp(&right.path));
        super::super::operational_output::write_canonical_json(
            &profile_dir.join("checksums.json"),
            &super::super::operational_output::ChecksumsFile {
                schema_version: "1".into(),
                profile_id: profile.id.clone(),
                entries,
            },
        )?;
    }
    let input = staging.join("assembled/source-closure-input.json");
    let input_parent = input
        .parent()
        .ok_or_else(|| io::Error::other("assembled input path has no parent"))?;
    fs::create_dir_all(input_parent)?;
    fs::write(&input, input_value.canonical_json_bytes()?)?;
    Ok(StagingFixture {
        root: fixture.root,
        staging,
        input,
    })
}

fn rewrite_value(value: &mut serde_json::Value, run_id: &str) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, value) in map.iter_mut() {
                if key == "capture_id"
                    && let serde_json::Value::String(capture_id) = value
                {
                    let suffix = capture_id
                        .splitn(CAPTURE_ID_PARTS, "::")
                        .nth(2)
                        .unwrap_or("capture");
                    *capture_id = format!("run::{run_id}::{suffix}");
                }
                rewrite_value(value, run_id);
            }
        }
        serde_json::Value::Array(values) => values
            .iter_mut()
            .for_each(|value| rewrite_value(value, run_id)),
        _ => {}
    }
}

fn rewrite_raw_paths(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, value) in map.iter_mut() {
                if key == "path"
                    && let serde_json::Value::String(path) = value
                    && let Some(stripped) = path.strip_prefix("raw/")
                {
                    *path = stripped.into();
                }
                rewrite_raw_paths(value);
            }
        }
        serde_json::Value::Array(values) => values.iter_mut().for_each(rewrite_raw_paths),
        _ => {}
    }
}

fn profile_evidence(
    profile: &super::super::operational_input::ProfileProbeInput,
) -> Vec<super::super::operational_input::EvidenceRef> {
    let mut values = vec![
        profile.rustc_vv_raw.clone(),
        profile.rustc_cfg_raw.clone(),
        profile.cargo_resolution_raw.clone(),
        profile.cargo_lock_raw.clone(),
        profile.cfg_edge_probe.clone(),
    ];
    values.extend(profile.source_tree.clone());
    values
}

fn unique_root(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| io::Error::other(format!("system clock is before Unix epoch: {error}")))?
        .as_nanos();
    Ok(std::env::temp_dir().join(format!(
        "kpc-source-closure-contract-{name}-{}-{}",
        std::process::id(),
        timestamp
    )))
}

#[test]
fn unknown_profile_artifact_directory_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = staging_fixture("unknown-profile")?;
    fs::create_dir(fixture.staging.join("profiles/unknown"))?;
    let result = validate_profile_layout(&fixture.staging, "run-1");
    fs::remove_dir_all(fixture.root)?;
    assert!(result.is_err());
    Ok(())
}

#[test]
fn cross_run_profile_evidence_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = staging_fixture("cross-run")?;
    let profile = fixture.staging.join("profiles/macos-latest/probe.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&profile)?)?;
    value["rustc_vv_raw"]["capture_id"] = serde_json::json!("run::other::profile::macos");
    let input: ProfileProbeInput = serde_json::from_value(value)?;
    fs::remove_file(&profile)?;
    super::super::operational_output::write_canonical_json(&profile, &input)?;
    let result = validate_profile_layout(&fixture.staging, "run-1");
    fs::remove_dir_all(fixture.root)?;
    assert!(result.is_err());
    Ok(())
}

#[test]
fn publication_requires_validation_receipt() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = staging_fixture("before-validation")?;
    let verified = SourceClosureInputLoaderVerifier::load(&fixture.input)?;
    let result = publish(
        &fixture.staging,
        &fixture.input,
        &verified,
        &unique_root("before-validation")?,
    );
    fs::remove_dir_all(fixture.root)?;
    assert!(result.is_err());
    Ok(())
}

#[test]
fn publication_requires_materialized_source_closure_artifact()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = staging_fixture("missing-materialized")?;
    let verified = SourceClosureInputLoaderVerifier::load(&fixture.input)?;
    write_validation_receipt(&fixture.staging, &fixture.input, &verified)?;
    let result = publish(
        &fixture.staging,
        &fixture.input,
        &verified,
        &unique_root("missing-materialized")?,
    );
    fs::remove_dir_all(fixture.root)?;
    assert!(matches!(
        result,
        Err(message) if message.contains("materialized source-closure artifact is required")
    ));
    Ok(())
}

#[test]
fn atomic_publish_and_existing_mismatch_are_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = staging_fixture("atomic")?;
    let verified = SourceClosureInputLoaderVerifier::load(&fixture.input)?;
    write_validation_receipt(&fixture.staging, &fixture.input, &verified)?;
    write_materialized_source_closure(&fixture.staging)?;
    validate_validation_receipt(&fixture.staging, &fixture.input)?;
    let canonical_root = unique_root("atomic")?;
    let published = publish(&fixture.staging, &fixture.input, &verified, &canonical_root)?;
    assert_eq!(
        published,
        canonical_root.join(format!(
            "v0-1-0/source-closure-input/{}",
            crate::source_closure::operational_input::FIXED_KATANA_REVISION
        ))
    );
    assert!(published.join("source-closure-input.json").is_file());
    assert!(published.join("artifacts/source-closure.json").is_file());
    assert!(!published.join(".tmp").exists());

    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(published.join("source-closure-input.json"))?)?;
    value["root"]["generated_at_utc"] = serde_json::json!("2026-08-22T00:00:00Z");
    let changed: SourceClosureInput = serde_json::from_value(value)?;
    fs::write(
        published.join("source-closure-input.json"),
        changed.canonical_json_bytes()?,
    )?;
    let result = publish(&fixture.staging, &fixture.input, &verified, &canonical_root);
    fs::remove_dir_all(fixture.root)?;
    fs::remove_dir_all(canonical_root)?;
    assert!(result.is_err());
    Ok(())
}

fn write_materialized_source_closure(staging: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let materialized = staging.join("materialized");
    fs::create_dir_all(&materialized)?;
    fs::write(materialized.join("source-closure.json"), b"source-closure")?;
    Ok(())
}
