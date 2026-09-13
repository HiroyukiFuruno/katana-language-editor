use crate::baseline::AUDIT_DOCUMENT;
use crate::matrix::{FEATURES, FeatureVerification, REQUIRED_FEATURE_COUNT};

const FEATURE_INVENTORY_COLUMN_COUNT: usize = 3;
const FEATURE_COLUMN: usize = 0;
const EVIDENCE_COLUMN: usize = 1;
const STATUS_COLUMN: usize = 2;

pub(super) fn validate_feature_inventory_rows() -> Result<(), String> {
    let rows = feature_inventory_rows();
    if rows.len() != REQUIRED_FEATURE_COUNT {
        return Err(format!(
            "audit feature inventory expected {REQUIRED_FEATURE_COUNT} rows, got {}",
            rows.len()
        ));
    }
    for (row, feature) in rows.into_iter().zip(FEATURES) {
        validate_feature_inventory_row(row, feature)?;
    }
    Ok(())
}

fn feature_inventory_rows() -> Vec<&'static str> {
    inventory_rows_in(AUDIT_DOCUMENT)
}

pub(super) fn inventory_rows_in(document: &str) -> Vec<&str> {
    document
        .split("## Required Feature Inventory")
        .nth(1)
        .and_then(|section| {
            section
                .split("| Feature | KatanA evidence | KLE v0.1.0 status |")
                .nth(1)
        })
        .and_then(|section| section.split("## Unresolved Parity Corrections").next())
        .map(|section| {
            section
                .lines()
                .filter(|line| line.starts_with("| ") && !line.contains("---"))
                .filter(|line| {
                    line.trim_matches('|').split('|').count() == FEATURE_INVENTORY_COLUMN_COUNT
                })
                .collect()
        })
        .unwrap_or_default()
}

fn validate_feature_inventory_row(
    row: &str,
    feature_verification: &FeatureVerification,
) -> Result<(), String> {
    let columns: Vec<&str> = row.trim_matches('|').split('|').map(str::trim).collect();
    if columns.len() != FEATURE_INVENTORY_COLUMN_COUNT {
        return Err(format!(
            "audit feature row must have {FEATURE_INVENTORY_COLUMN_COUNT} columns: {row}"
        ));
    }
    let feature = columns[FEATURE_COLUMN];
    let evidence = columns[EVIDENCE_COLUMN];
    let status = columns[STATUS_COLUMN];
    validate_nonempty_columns(feature, evidence, status, row)?;
    validate_feature_status(status, feature, feature_verification)
}

fn validate_nonempty_columns(
    feature: &str,
    evidence: &str,
    status: &str,
    row: &str,
) -> Result<(), String> {
    if feature.is_empty() || evidence.is_empty() || status.is_empty() {
        return Err(format!("audit feature row has an empty column: {row}"));
    }
    if !classification_is_recorded(status) {
        return Err(format!(
            "audit feature row must classify KLE/KUC/downstream ownership: {feature}"
        ));
    }
    Ok(())
}

fn validate_feature_status(
    status: &str,
    feature: &str,
    verification: &FeatureVerification,
) -> Result<(), String> {
    if verification.has_open_gap() {
        return validate_incomplete_feature(status, feature, verification.id);
    }
    validate_implemented_feature(status, feature)
}

fn validate_incomplete_feature(status: &str, feature: &str, id: &str) -> Result<(), String> {
    if !has_incomplete_status(status) && !correction_is_recorded(id) {
        return Err(format!(
            "audit feature row must preserve incomplete release status: {feature}"
        ));
    }
    if !has_explicit_blocker(status) && !correction_is_recorded(id) {
        return Err(format!(
            "audit feature row must keep the KatanA integration blocker explicit: {feature}"
        ));
    }
    Ok(())
}

fn validate_implemented_feature(status: &str, feature: &str) -> Result<(), String> {
    if is_reference_only_adapter_blocker(status) {
        return Ok(());
    }
    if !status.contains("Implemented.") {
        return Err(format!(
            "audit feature row must mark implemented feature as Implemented: {feature}"
        ));
    }
    if status.contains("not proven")
        || status.contains("not complete")
        || status.contains("missing")
    {
        return Err(format!(
            "implemented audit feature row must not keep blocker wording: {feature}"
        ));
    }
    Ok(())
}

fn is_reference_only_adapter_blocker(status: &str) -> bool {
    status.contains("Blocked.")
        && (status.contains("adapter path is planned only")
            || status.contains("no actual KatanA/KLE runnable evidence")
            || status.contains("planned KatanA adapter source")
            || status.contains("reference checkout lacks the planned adapter path")
            || status.contains("planned downstream adapter source/module/dependencies")
            || status.contains("planned adapter source/module/dependencies"))
}

fn has_incomplete_status(status: &str) -> bool {
    status.contains("Partial.") || status.contains("Blocked.")
}

fn has_explicit_blocker(status: &str) -> bool {
    [
        "not proven",
        "not complete",
        "missing",
        "unimplemented",
        "unsupported",
        "absent",
        "lacks",
        "planned only",
        "do not exist",
    ]
    .iter()
    .any(|marker| status.contains(marker))
}

fn correction_is_recorded(feature_id: &str) -> bool {
    let Some(section) = AUDIT_DOCUMENT
        .split("## Unresolved Parity Corrections")
        .nth(1)
    else {
        return false;
    };
    section
        .split("## Release Blockers")
        .next()
        .is_some_and(|corrections| corrections.contains(feature_id))
}

fn classification_is_recorded(status: &str) -> bool {
    ["KLE", "egui", "Storybook", "downstream", "KUC", "neutral"]
        .iter()
        .any(|marker| status.contains(marker))
}

#[cfg(test)]
#[path = "baseline_inventory_tests.rs"]
mod tests;
