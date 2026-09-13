use crate::release_gate::ReleaseGateAudit;

fn assert_error_contains(result: Result<(), String>, expected: &str) -> Result<(), String> {
    match result {
        Ok(()) => Err(format!("expected error containing {expected}, got Ok")),
        Err(error) if error.contains(expected) => Ok(()),
        Err(error) => Err(format!("expected error containing {expected}, got {error}")),
    }
}

#[test]
fn source_closure_contract_rejects_missing_kuc_dependency_checkout() -> Result<(), String> {
    let lines = workflow_fixture().replace(
        "      - name: Checkout KUC dependency source",
        "      - name: Checkout missing KUC dependency source",
    );
    let lines = lines.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_source_closure_native_host_contract_from_lines(&lines);
    assert_error_contains(result, "Checkout KUC dependency source")
}

#[test]
fn source_closure_contract_rejects_kuc_checkout_missing_only_from_native_host_job()
-> Result<(), String> {
    let marker = "      - name: Checkout KUC dependency source";
    let mut fixture = workflow_fixture();
    let native_checkout = fixture
        .rfind(marker)
        .ok_or("native-host-e2e KUC checkout fixture is missing")?;
    fixture.replace_range(
        native_checkout..native_checkout + marker.len(),
        "      - name: Checkout missing KUC dependency source",
    );
    let lines = fixture.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_source_closure_native_host_contract_from_lines(&lines);
    assert_error_contains(result, "native-host-e2e job is missing")
}

#[test]
fn source_closure_contract_rejects_unpinned_kuc_dependency_checkout() -> Result<(), String> {
    let lines = workflow_fixture().replace("ref: v0.3.10", "ref: master");
    let lines = lines.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_source_closure_native_host_contract_from_lines(&lines);
    assert_error_contains(result, "ref: v0.3.10")
}

#[test]
fn source_closure_contract_rejects_missing_fixed_katana_dependency_fetch() -> Result<(), String> {
    let lines = workflow_fixture().replace(
        "      - name: Fetch fixed KatanA dependencies",
        "      - name: Fetch missing KatanA dependencies",
    );
    let lines = lines.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_source_closure_native_host_contract_from_lines(&lines);
    assert_error_contains(result, "Fetch fixed KatanA dependencies")
}

#[test]
fn source_closure_contract_rejects_default_artifact_directory() -> Result<(), String> {
    let lines = workflow_fixture().replace(
        "artifacts/v0-1-0/source-closure-input/4f6a6287c650a38633c7baeb544a92e739c68567/artifacts",
        "target/katana-parity-check/artifacts",
    );
    let lines = lines.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_source_closure_native_host_contract_from_lines(&lines);
    assert_error_contains(result, "canonical artifact directory")
}

#[test]
fn source_closure_contract_rejects_single_text_edit_seed() -> Result<(), String> {
    let lines = workflow_fixture().replace(
        "--seed-manifest docs/v0-1-0-source-closure-roots.json",
        "--seed-path crates/katana-ui/src/views/panels/editor/text_edit.rs",
    );
    let lines = lines.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_source_closure_native_host_contract_from_lines(&lines);
    assert_error_contains(
        result,
        "--seed-manifest docs/v0-1-0-source-closure-roots.json",
    )
}

#[test]
fn source_closure_recipe_rejects_the_legacy_single_seed() -> Result<(), String> {
    let lines = crate::release_gate_sources::JUSTFILE.replace(
        "--seed-manifest \"{{REPO_ROOT}}/docs/v0-1-0-source-closure-roots.json\"",
        "--seed-path \"crates/katana-ui/src/views/panels/editor/text_edit.rs\"",
    );
    let lines = lines.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_source_closure_input_recipes_from_lines(&lines);
    assert_error_contains(result, "--seed-manifest")
}

#[test]
fn source_closure_contract_rejects_missing_requirement_source_aliases() -> Result<(), String> {
    let lines = workflow_fixture().replace(
        "          --requirement-source-aliases docs/v0-1-0-editor-requirement-source-aliases.json\n",
        "",
    );
    let lines = lines.lines().collect::<Vec<_>>();
    let result = ReleaseGateAudit::validate_source_closure_native_host_contract_from_lines(&lines);
    assert_error_contains(result, "--requirement-source-aliases")
}

fn workflow_fixture() -> String {
    crate::release_gate_sources::SOURCE_CLOSURE_WORKFLOW.to_string()
}
