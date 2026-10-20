use super::{
    FixtureBuilder, OpaqueTransitOutcome, empty_receipt, evidence, failure, validate_integrity,
};

#[test]
fn rejects_no_mutation_presentation_drift() -> Result<(), String> {
    let fixture = FixtureBuilder::root()?;
    let mut evidence = evidence(&fixture)?;
    let before = empty_receipt();
    let mut after = before.clone();
    after.presentation_revision = 2;
    evidence.records[0].outcome = OpaqueTransitOutcome::NoMutation {
        before_receipt: before,
        after_receipt: after,
    };
    failure(
        validate_integrity(fixture.path(), &evidence),
        "presentation/state-stable",
    )
}
