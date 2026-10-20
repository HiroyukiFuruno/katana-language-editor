use super::super::kle_release_evidence::{OpaqueTransitOutcome, PublicKucReceipt};

pub(super) fn validate_outcome(outcome: &OpaqueTransitOutcome) -> Result<(), String> {
    match outcome {
        OpaqueTransitOutcome::SingleForwardedBatch { receipt } => {
            validate_receipt(receipt)?;
            if receipt.event_cardinality == 0 {
                return Err(
                    "single forwarded KUC batch must retain at least one KUC event".to_string(),
                );
            }
        }
        OpaqueTransitOutcome::NoMutation {
            before_receipt,
            after_receipt,
        } => {
            validate_receipt(before_receipt)?;
            validate_receipt(after_receipt)?;
            if before_receipt.root_identity != after_receipt.root_identity
                || before_receipt.presentation_revision != after_receipt.presentation_revision
                || before_receipt.state_revision != after_receipt.state_revision
                || before_receipt.event_cardinality != 0
                || after_receipt.event_cardinality != 0
            {
                return Err(
                    "no-mutation KUC receipt pair is not presentation/state-stable and event-empty"
                        .to_string(),
                );
            }
        }
    }
    Ok(())
}
fn validate_receipt(receipt: &PublicKucReceipt) -> Result<(), String> {
    if !receipt.consumed_once {
        return Err("KUC receipt was not consumed exactly once".to_string());
    }
    for (name, value) in [
        ("root_identity", receipt.root_identity.as_str()),
        ("record_hash", receipt.record_hash.as_str()),
        ("paint_plan_hash", receipt.paint_plan_hash.as_str()),
        (
            "accessibility_snapshot_hash",
            receipt.accessibility_snapshot_hash.as_str(),
        ),
        (
            "correlation_fingerprint",
            receipt.correlation_fingerprint.as_str(),
        ),
        (
            "event_batch_fingerprint",
            receipt.event_batch_fingerprint.as_str(),
        ),
    ] {
        super::nonempty(name, value)?;
    }
    Ok(())
}
