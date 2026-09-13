use crate::matrix::{DOWNSTREAM_CHECK, FeatureVerification};

const FEATURES_REQUIRING_DOWNSTREAM_CHECK: &[&str] = &[
    "text-editing",
    "document-scoped-undo",
    "line-gutter",
    "diagnostics-problems",
    "authoring-toolbar",
    "code-block-menu",
    "context-menu-actions",
    "image-ingest",
    "document-search",
    "scroll-sync",
    "view-modes",
    "select-and-jump",
    "dirty-save-refresh",
    "multi-document-state",
];

pub(crate) struct DownstreamEvidenceAudit;

impl DownstreamEvidenceAudit {
    pub(crate) fn validate(feature: &FeatureVerification) -> Result<(), String> {
        if FEATURES_REQUIRING_DOWNSTREAM_CHECK.contains(&feature.id)
            && !has_automated_check(feature, DOWNSTREAM_CHECK)
        {
            return Err(format!(
                "{} must include {DOWNSTREAM_CHECK} in automated evidence",
                feature.id
            ));
        }
        Ok(())
    }
}

fn has_automated_check(feature: &FeatureVerification, expected: &str) -> bool {
    feature.automated_checks.contains(&expected)
}
