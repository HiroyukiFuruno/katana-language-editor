use super::consumer_artifact_model::{
    PLAN_SCHEMA, SourceLeafArtifactPlan, current_profile_id, validate_plan,
};
use super::consumer_artifact_provenance::ArtifactProvenance;

pub(super) fn provenance() -> ArtifactProvenance {
    ArtifactProvenance {
        run_id: "test-run".into(),
        source_closure_sha256: format!("sha256:{}", "a".repeat(64)),
        profile_fingerprint: format!("sha256:{}", "b".repeat(64)),
    }
}

fn plan_json() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut plan = serde_json::to_value(provenance())?;
    plan["schema"] = PLAN_SCHEMA.into();
    plan["profile_id"] = current_profile_id()?.into();
    plan["stages"] = serde_json::json!([{"leaf_id":"leaf-a","interaction":"text_input"}]);
    Ok(plan)
}

#[test]
fn source_leaf_provenance_json_roundtrip_preserves_values() -> Result<(), Box<dyn std::error::Error>>
{
    let original = provenance();
    let json = serde_json::to_vec(&original)?;
    let decoded: ArtifactProvenance = serde_json::from_slice(&json)?;
    assert_eq!(decoded, original);
    decoded.validate()?;
    let plan: SourceLeafArtifactPlan = serde_json::from_slice(&serde_json::to_vec(&plan_json()?)?)?;
    assert_eq!(validate_plan(plan)?.provenance, original);
    Ok(())
}

#[test]
fn source_leaf_provenance_json_rejects_invalid_run_ids() -> Result<(), Box<dyn std::error::Error>> {
    for run_id in ["", " ", "run id", "run\tid", "run\nid", "run\u{3000}id"] {
        let mut json = plan_json()?;
        json["run_id"] = run_id.into();
        let plan = serde_json::from_slice(&serde_json::to_vec(&json)?)?;
        assert!(validate_plan(plan).is_err(), "accepted run_id: {run_id:?}");
    }
    Ok(())
}

#[test]
fn source_leaf_provenance_json_rejects_malformed_digests() -> Result<(), Box<dyn std::error::Error>>
{
    let hex = "a".repeat(64);
    for field in ["source_closure_sha256", "profile_fingerprint"] {
        for invalid in [
            String::new(),
            hex.clone(),
            format!("SHA256:{hex}"),
            format!("sha256:{}", "A".repeat(64)),
            format!("sha256:{}", "g".repeat(64)),
            format!("sha256:{}", "a".repeat(63)),
            format!("sha256:{}", "a".repeat(65)),
            format!("sha256:{hex} "),
        ] {
            let mut json = plan_json()?;
            json[field] = invalid.into();
            let plan = serde_json::from_slice(&serde_json::to_vec(&json)?)?;
            assert!(validate_plan(plan).is_err(), "accepted malformed {field}");
        }
    }
    Ok(())
}

#[test]
fn source_leaf_provenance_json_requires_string_fields() -> Result<(), Box<dyn std::error::Error>> {
    for field in ["run_id", "source_closure_sha256", "profile_fingerprint"] {
        let mut json = plan_json()?;
        json.as_object_mut()
            .ok_or("plan must be an object")?
            .remove(field);
        assert!(
            serde_json::from_slice::<SourceLeafArtifactPlan>(&serde_json::to_vec(&json)?).is_err()
        );
        for invalid in [serde_json::Value::Null, serde_json::json!(123)] {
            json[field] = invalid;
            assert!(
                serde_json::from_slice::<SourceLeafArtifactPlan>(&serde_json::to_vec(&json)?)
                    .is_err()
            );
        }
    }
    Ok(())
}

#[test]
fn source_leaf_provenance_json_rejects_unknown_fields() -> Result<(), Box<dyn std::error::Error>> {
    let mut json = plan_json()?;
    json["unknown"] = "unexpected".into();
    assert!(serde_json::from_slice::<SourceLeafArtifactPlan>(&serde_json::to_vec(&json)?).is_err());
    let mut json = plan_json()?;
    json["stages"][0]["unknown"] = "unexpected".into();
    assert!(serde_json::from_slice::<SourceLeafArtifactPlan>(&serde_json::to_vec(&json)?).is_err());
    let mut json = serde_json::to_value(provenance())?;
    json["unknown"] = "unexpected".into();
    assert!(serde_json::from_slice::<ArtifactProvenance>(&serde_json::to_vec(&json)?).is_err());
    Ok(())
}
