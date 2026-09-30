use crate::baseline::AUDIT_DOCUMENT;
use crate::baseline::inventory::inventory_rows_in;

const FEATURE_COLUMN: usize = 0;
const STATUS_COLUMN: usize = 2;

pub(super) fn validate_historical_adapter_claims_are_not_live_evidence() -> Result<(), String> {
    validate_reference_only_inventory_rows(AUDIT_DOCUMENT)
}

fn validate_reference_only_inventory_rows(document: &str) -> Result<(), String> {
    if !is_revalidated_reference_document(document) {
        return Ok(());
    }
    reject_live_adapter_headlines(document)?;
    for row in inventory_rows_in(document) {
        validate_reference_only_row(row)?;
    }
    Ok(())
}

fn is_revalidated_reference_document(document: &str) -> bool {
    document.contains("## Evidence Revalidation (2026-08-13)")
        && document.contains("Current authoritative status")
}

fn reject_live_adapter_headlines(document: &str) -> Result<(), String> {
    if document.contains("Actual KatanA adapter validation now runs")
        || document.contains("The real KatanA adapter test")
    {
        return Err(
            "audit document promotes a planned KatanA adapter as live evidence".to_string(),
        );
    }
    Ok(())
}

fn validate_reference_only_row(row: &str) -> Result<(), String> {
    let columns: Vec<&str> = row.trim_matches('|').split('|').map(str::trim).collect();
    let status = columns[STATUS_COLUMN];
    reject_implemented_status(status, columns[FEATURE_COLUMN])?;
    reject_positive_downstream_claim(status, columns[FEATURE_COLUMN])?;
    reject_passed_planned_adapter(row, status)
}

fn reject_implemented_status(status: &str, feature: &str) -> Result<(), String> {
    if status.contains("Implemented.") {
        return Err(format!(
            "reference-only audit inventory row must not be Implemented: {feature}"
        ));
    }
    Ok(())
}

fn reject_positive_downstream_claim(status: &str, feature: &str) -> Result<(), String> {
    let claims = [
        "Real KatanA integration now proves",
        "real KatanA integration now proves",
        "actual downstream adapter proves",
        "real KatanA adapter test",
    ];
    if claims.iter().any(|claim| status.contains(claim)) {
        return Err(format!(
            "reference-only audit inventory row contains a positive downstream claim: {feature}"
        ));
    }
    Ok(())
}

fn reject_passed_planned_adapter(row: &str, status: &str) -> Result<(), String> {
    if (row.contains("kle_downstream_adapter") || status.contains("editor_kle_downstream_adapter"))
        && status.contains("PASS")
    {
        return Err(format!(
            "audit document marks planned adapter evidence as passed: {row}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn historical_adapter_claims_cannot_be_live_evidence() -> Result<(), String> {
        validate_historical_adapter_claims_are_not_live_evidence()
    }

    #[test]
    fn reference_only_inventory_rejects_implemented_status() {
        let document = "## Evidence Revalidation (2026-08-13)\n### Current authoritative status\n## Required Feature Inventory\n| Feature | KatanA evidence | KLE v0.1.0 status |\n| --- | --- | --- |\n| stale | `kle_downstream_adapter.rs` | Implemented. KLE downstream evidence exists. |\n## Unresolved Parity Corrections\n";
        assert!(matches!(
            validate_reference_only_inventory_rows(document).as_ref(),
            Err(error) if error.contains("must not be Implemented")
        ));
    }

    #[test]
    fn reference_only_inventory_rejects_positive_downstream_claim() {
        let document = "## Evidence Revalidation (2026-08-13)\n### Current authoritative status\n## Required Feature Inventory\n| Feature | KatanA evidence | KLE v0.1.0 status |\n| --- | --- | --- |\n| stale | `kle_downstream_adapter.rs` | Blocked. Real KatanA integration now proves this feature. |\n## Unresolved Parity Corrections\n";
        assert!(matches!(
            validate_reference_only_inventory_rows(document).as_ref(),
            Err(error) if error.contains("positive downstream claim")
        ));
    }
}
