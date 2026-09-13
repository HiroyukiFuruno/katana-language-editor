use super::*;

#[test]
fn missing_windows_and_linux_profiles_reject() -> TestResult {
    for missing in ["windows-latest", "ubuntu-latest"] {
        let fixture = fixture(missing)?;
        mutate_input(&fixture, |value| {
            let profiles =
                option_result(value["profile_probes"].as_array(), "profiles array missing")?;
            value["profile_probes"] = serde_json::Value::Array(
                profiles
                    .iter()
                    .filter(|probe| probe["id"] != missing)
                    .cloned()
                    .collect(),
            );
            Ok(())
        })?;
        reject(fixture)?;
    }
    Ok(())
}

#[test]
fn macos_evidence_reuse_under_windows_rejects() -> TestResult {
    let fixture = fixture("profile-reuse")?;
    mutate_input(&fixture, |value| {
        let mac = value["profile_probes"][0]["rustc_vv_raw"].clone();
        value["profile_probes"][1]["rustc_vv_raw"] = mac;
        Ok(())
    })?;
    reject(fixture)?;
    Ok(())
}

#[test]
fn profile_order_and_duplicate_reject() -> TestResult {
    for duplicate in [false, true] {
        let fixture = fixture(if duplicate { "duplicate" } else { "order" })?;
        mutate_input(&fixture, |value| {
            let profiles = option_result(
                value["profile_probes"].as_array_mut(),
                "profiles array missing",
            )?;
            if duplicate {
                profiles[1]["id"] = serde_json::json!("macos-latest");
                profiles[1]["runner_label"] = serde_json::json!("macos-latest");
            } else {
                profiles.swap(0, 1);
            }
            Ok(())
        })?;
        reject(fixture)?;
    }
    Ok(())
}

#[test]
fn profile_revision_is_required_and_must_be_fixed_and_root_consistent() -> TestResult {
    let missing = fixture("profile-revision-missing")?;
    mutate_input(&missing, |value| {
        let profile = option_result(
            value["profile_probes"][0].as_object_mut(),
            "profile object missing",
        )?;
        profile.remove("katana_revision");
        Ok(())
    })?;
    reject(missing)?;

    let nonfixed = fixture("profile-revision-nonfixed")?;
    mutate_input(&nonfixed, |value| {
        value["profile_probes"][0]["katana_revision"] = serde_json::json!("deadbeef");
        Ok(())
    })?;
    reject(nonfixed)?;

    let mismatch = fixture("profile-revision-root-mismatch")?;
    mutate_input(&mismatch, |value| {
        value["profile_probes"][0]["katana_revision"] = value["root"]["katana_revision"].clone();
        value["profile_probes"][1]["katana_revision"] =
            serde_json::json!("4f6a6287c650a38633c7baeb544a92e739c68567");
        value["root"]["katana_revision"] = serde_json::json!("deadbeef");
        Ok(())
    })?;
    reject(mismatch)?;
    Ok(())
}
