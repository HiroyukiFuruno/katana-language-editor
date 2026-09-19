use super::operational_fixture::fixture;
use super::operational_test_support::{TestResult, mutate_input, option_result, reject};

#[test]
fn profile_raw_evidence_mismatches_reject() -> TestResult {
    let fields = [
        "rustc_vv_raw",
        "rustc_cfg_raw",
        "cargo_resolution_raw",
        "cargo_lock_raw",
        "source_tree",
        "cfg_edge_probe",
    ];
    for field in fields {
        let fixture = fixture(field)?;
        mutate_input(&fixture, |value| {
            let hash = serde_json::json!(
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            );
            if field == "source_tree" {
                value["profile_probes"][0][field][0]["sha256"] = hash;
            } else {
                value["profile_probes"][0][field]["sha256"] = hash;
            }
            Ok(())
        })?;
        reject(fixture)?;
    }
    Ok(())
}

#[test]
fn profile_runner_command_and_edge_contract_mismatches_reject() -> TestResult {
    let cases = [
        "runner",
        "host",
        "rustc-command",
        "cfg-command",
        "edge-omission",
    ];
    for case in cases {
        let fixture = fixture(case)?;
        mutate_input(&fixture, |value| {
            match case {
                "runner" => {
                    value["profile_probes"][0]["runner_label"] =
                        serde_json::json!("windows-latest");
                }
                "host" => {
                    value["profile_probes"][0]["rustc_host_triple"] =
                        serde_json::json!("wrong-host");
                }
                "rustc-command" => {
                    value["profile_probes"][0]["rustc_vv_raw"]["command_or_source"] =
                        serde_json::json!("rustc --version");
                }
                "cfg-command" => {
                    value["profile_probes"][0]["rustc_cfg_raw"]["command_or_source"] =
                        serde_json::json!("manual cfg");
                }
                "edge-omission" => {
                    value["profile_probes"][0]["active_edge_ids"] =
                        serde_json::json!(["edge:macos-latest:other"]);
                }
                _ => {}
            }
            Ok(())
        })?;
        reject(fixture)?;
    }
    Ok(())
}

#[test]
fn root_raw_evidence_sha_and_schema_mismatch_reject() -> TestResult {
    let cases = [
        ("revision", "katana_revision"),
        ("tree", "katana_tree"),
        ("external", "katana_external_ui"),
        ("user", "user_mandated_extensions"),
        ("aliases", "requirement_source_aliases"),
        ("kle", "kle_tree"),
        ("kuc", "kuc_tree"),
        ("generator", "generator_schema"),
    ];
    for (name, key) in cases {
        let fixture = fixture(name)?;
        mutate_input(&fixture, |value| {
            let hash = serde_json::json!(
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            );
            if matches!(name, "tree" | "external" | "kle" | "kuc") {
                value["root"]["evidence"][key][0]["sha256"] = hash;
            } else {
                value["root"]["evidence"][key]["sha256"] = hash;
            }
            Ok(())
        })?;
        reject(fixture)?;
    }
    Ok(())
}

#[test]
fn schema_version_and_placeholder_evidence_metadata_reject() -> TestResult {
    let schema_fixture = fixture("schema-version")?;
    mutate_input(&schema_fixture, |value| {
        value["input_schema_version"] = serde_json::json!(2);
        Ok(())
    })?;
    reject(schema_fixture)?;

    let placeholder_fixture = fixture("placeholder-evidence")?;
    mutate_input(&placeholder_fixture, |value| {
        value["root"]["evidence"]["katana_revision"]["capture_id"] =
            serde_json::json!("placeholder");
        Ok(())
    })?;
    reject(placeholder_fixture)?;
    Ok(())
}

#[test]
fn root_evidence_and_revision_mismatches_reject() -> TestResult {
    let fields = [
        "katana_tree_fingerprint",
        "katana_external_ui_fingerprint",
        "user_mandated_extensions_fingerprint",
        "requirement_source_aliases_fingerprint",
        "kle_tree_fingerprint",
        "kuc_tree_fingerprint",
        "generator_fingerprint",
    ];
    for field in fields {
        let fixture = fixture(field)?;
        mutate_input(&fixture, |value| {
            value["root"][field] = serde_json::json!(
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            );
            Ok(())
        })?;
        reject(fixture)?;
    }
    let fixture = fixture("revision")?;
    mutate_input(&fixture, |value| {
        value["root"]["katana_revision"] = serde_json::json!("deadbeef");
        Ok(())
    })?;
    reject(fixture)?;
    Ok(())
}

#[test]
fn captured_requirement_source_alias_bytes_must_match_the_checker_ledger() -> TestResult {
    let fixture = fixture("aliases-content")?;
    let bytes = b"{}\n";
    std::fs::write(
        fixture
            .root
            .join("input/raw/provenance/editor-requirement-source-aliases.json"),
        bytes,
    )?;
    let hash = super::super::fingerprint::sha256_hex(bytes);
    mutate_input(&fixture, |value| {
        value["root"]["requirement_source_aliases_fingerprint"] = serde_json::json!(hash);
        value["root"]["evidence"]["requirement_source_aliases"]["sha256"] = serde_json::json!(hash);
        Ok(())
    })?;
    reject(fixture)
}

#[test]
fn invalid_path_exit_status_and_placeholder_evidence_reject() -> TestResult {
    for (name, path) in [("absolute", "/tmp/evidence"), ("traversal", "../evidence")] {
        let fixture = fixture(name)?;
        mutate_input(&fixture, |value| {
            value["root"]["evidence"]["katana_revision"]["path"] = serde_json::json!(path);
            Ok(())
        })?;
        reject(fixture)?;
    }
    let missing_fixture = fixture("missing")?;
    mutate_input(&missing_fixture, |value| {
        value["root"]["evidence"]["katana_revision"]["path"] = serde_json::json!("raw/missing");
        Ok(())
    })?;
    reject(missing_fixture)?;
    let exit_fixture = fixture("exit")?;
    mutate_input(&exit_fixture, |value| {
        value["root"]["evidence"]["katana_revision"]["exit_status"] = serde_json::json!(1);
        Ok(())
    })?;
    reject(exit_fixture)?;
    let placeholder_fixture = fixture("placeholder")?;
    mutate_input(&placeholder_fixture, |value| {
        value["root"]["katana_tree_fingerprint"] = serde_json::json!(
            "sha256:0000000000000000000000000000000000000000000000000000000000000000"
        );
        Ok(())
    })?;
    reject(placeholder_fixture)?;
    Ok(())
}

#[test]
fn katana_tree_source_paths_reject_duplicate_traversal_and_non_rust() -> TestResult {
    let duplicate = fixture("tree-duplicate")?;
    mutate_input(&duplicate, |value| {
        value["root"]["evidence"]["katana_tree"][1]["source_path"] =
            value["root"]["evidence"]["katana_tree"][0]["source_path"].clone();
        Ok(())
    })?;
    reject(duplicate)?;

    for path in [
        "../foreign.rs",
        "/foreign.rs",
        "foreign\\source.rs",
        "foreign.txt",
    ] {
        let fixture = fixture("tree-invalid-path")?;
        mutate_input(&fixture, |value| {
            value["root"]["evidence"]["katana_tree"][0]["source_path"] = serde_json::json!(path);
            Ok(())
        })?;
        reject(fixture)?;
    }
    Ok(())
}

#[test]
fn timestamp_only_changes_preserve_identity() -> TestResult {
    let first = fixture("timestamp-first")?;
    let first_verified =
        super::super::operational_loader::SourceClosureInputLoaderVerifier::load(&first.input_json)
            .map_err(super::operational_test_support::test_error)?;
    let first_identity = first_verified.identity_fingerprint().to_string();
    super::operational_test_support::cleanup_dir(first.root)?;

    let second = fixture("timestamp-second")?;
    mutate_input(&second, |value| {
        value["root"]["generated_at_utc"] = serde_json::json!("2026-08-21T12:34:56Z");
        Ok(())
    })?;
    let second_verified = super::super::operational_loader::SourceClosureInputLoaderVerifier::load(
        &second.input_json,
    )
    .map_err(super::operational_test_support::test_error)?;
    assert_eq!(first_identity, second_verified.identity_fingerprint());
    super::operational_test_support::cleanup_dir(second.root)?;
    Ok(())
}

#[test]
fn unknown_fields_and_direct_unverified_materialization_path_reject() -> TestResult {
    let fixture = fixture("unknown-field")?;
    mutate_input(&fixture, |value| {
        value["root"]["unexpected"] = serde_json::json!(true);
        Ok(())
    })?;
    reject(fixture)?;

    let source = include_str!("materializer.rs");
    assert!(source.contains("verified: &VerifiedSourceClosureInput"));
    assert!(!source.contains("input: &SourceClosureInput"));
    Ok(())
}

#[test]
fn mixed_capture_runs_reject() -> TestResult {
    let fixture = fixture("mixed-run")?;
    mutate_input(&fixture, |value| {
        value["profile_probes"][1]["rustc_vv_raw"]["capture_id"] =
            serde_json::json!("run::different-run::profile::windows::rustc-vv");
        Ok(())
    })?;
    reject(fixture)?;
    Ok(())
}

#[test]
fn noncanonical_input_json_rejects_before_evidence_validation() -> TestResult {
    let fixture = fixture("noncanonical")?;
    let bytes = std::fs::read(&fixture.input_json)?;
    let truncated = option_result(
        bytes.get(..bytes.len().saturating_sub(1)),
        "input fixture is empty",
    )?;
    std::fs::write(&fixture.input_json, truncated)?;
    reject(fixture)?;
    Ok(())
}
