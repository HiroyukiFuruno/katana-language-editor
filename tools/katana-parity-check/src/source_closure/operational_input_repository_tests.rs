use super::operational_fixture::fixture;
use super::operational_test_support::{TestResult, mutate_input, reject};

#[test]
fn kle_and_kuc_revision_and_cleanliness_mismatches_reject() -> TestResult {
    let cases = [
        ("kle_revision", serde_json::json!("deadbeef")),
        ("kle_worktree_clean", serde_json::json!(false)),
        ("kuc_revision", serde_json::json!("deadbeef")),
        ("kuc_worktree_clean", serde_json::json!(false)),
    ];
    for (field, value) in cases {
        let fixture = fixture(field)?;
        mutate_input(&fixture, |input| {
            input["root"][field] = value.clone();
            Ok(())
        })?;
        reject(fixture)?;
    }

    for evidence_key in [
        "kle_revision",
        "kle_worktree_status",
        "kuc_revision",
        "kuc_worktree_status",
    ] {
        let fixture = fixture(evidence_key)?;
        mutate_input(&fixture, |input| {
            input["root"]["evidence"][evidence_key]["command_or_source"] =
                serde_json::json!("untrusted command");
            Ok(())
        })?;
        reject(fixture)?;
    }
    Ok(())
}
