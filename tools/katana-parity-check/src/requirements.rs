pub(crate) const REQUIREMENTS_DOCUMENT: &str =
    include_str!("../../../docs/v0-1-0-editor-requirements.md");

pub(crate) const REQUIREMENT_SECTIONS: &[&str] = &[
    "## v0.1.0 Done Definition",
    "## 責務境界",
    "## Editor Parity Requirements",
    "## Automation Matrix",
    "## Current Verification State",
];

pub(crate) const REQUIREMENT_SOURCE_REFERENCES: &[&str] = &[
    "openspec/changes/v0-1-0-language-editor-extraction",
    "docs/v0-1-0-katana-editor-full-parity-audit.md",
    "docs/v0-1-0-katana-editor-migration.md",
    "katana-document-viewer/tools/kdv-storybook",
    "katana-ui-core/crates/katana-ui-core",
];

pub(crate) const REQUIREMENT_AUTOMATION_CHECKS: &[&str] = &[
    "just kuc-contract-check",
    "just katana-parity-check",
    "just katana-downstream-check",
    "KLE-owned actual KatanA host-E2E",
    "just storybook-contract-check",
    "just check",
];

pub(crate) const REQUIREMENT_CATEGORY_HEADINGS: &[&str] = &[
    "### 1. Text editing",
    "### 2. Cursor and selection",
    "### 3. Line numbers and gutter",
    "### 4. Search",
    "### 5. Syntax highlight",
    "### 6. Diagnostics and decorations",
    "### 7. Scroll and viewport",
    "### 8. Shortcuts and commands",
    "### 9. Clipboard and paste",
    "### 10. Theme, i18n, typography, spacing",
    "### 11. Emoji and IME",
    "### 12. Storybook",
    "### 13. KatanA editor full parity",
];

pub(crate) struct RequirementsDocumentAudit;

impl RequirementsDocumentAudit {
    pub(crate) fn validate() -> Result<(), String> {
        validate_requirements_contains_all(REQUIREMENT_SECTIONS, "requirements section")?;
        validate_requirements_contains_all(
            REQUIREMENT_SOURCE_REFERENCES,
            "requirements source reference",
        )?;
        validate_requirements_contains_all(
            REQUIREMENT_AUTOMATION_CHECKS,
            "requirements automation check",
        )?;
        validate_requirements_contains_all(
            REQUIREMENT_CATEGORY_HEADINGS,
            "requirements category heading",
        )
    }
}

fn validate_requirements_contains_all(needles: &[&str], label: &str) -> Result<(), String> {
    for needle in needles {
        if !REQUIREMENTS_DOCUMENT.contains(needle) {
            return Err(format!(
                "requirements document is missing {label}: {needle}"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requirements_document_records_editor_requirements_and_gates() -> Result<(), String> {
        RequirementsDocumentAudit::validate()
    }
}
