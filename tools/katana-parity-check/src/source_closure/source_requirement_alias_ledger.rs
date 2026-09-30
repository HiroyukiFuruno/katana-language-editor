use serde::Deserialize;

pub(super) const REQUIREMENTS: &str =
    include_str!("../../../../docs/v0-1-0-editor-requirements.md");
pub(super) const SOURCE_UNIVERSE: &str =
    include_str!("../../../../docs/v0-1-0-katana-editor-source-universe.md");
pub(super) const ROOT_MANIFEST: &str =
    include_str!("../../../../docs/v0-1-0-source-closure-roots.json");
pub(super) const ALIAS_LEDGER: &str =
    include_str!("../../../../docs/v0-1-0-editor-requirement-source-aliases.json");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AliasLedger {
    pub(super) schema_version: String,
    pub(super) katana_revision: String,
    pub(super) entries: Vec<AliasEntry>,
    pub(super) non_katana_references: Vec<ExternalReference>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct AliasEntry {
    pub(crate) requirement_id: String,
    pub(crate) short_reference: String,
    pub(crate) source_path: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ExternalReference {
    pub(super) requirement_id: String,
    pub(super) short_reference: String,
    pub(super) owner: String,
    pub(super) reason: String,
}

#[derive(Deserialize)]
pub(super) struct SourceRootManifest {
    pub(super) directory_roots: Vec<String>,
    pub(super) file_roots: Vec<String>,
}

pub(crate) struct RequirementSourceAliasLedger;

impl RequirementSourceAliasLedger {
    pub(crate) fn verified_alias_entries() -> Result<Vec<AliasEntry>, String> {
        Self::validated_entries(REQUIREMENTS, SOURCE_UNIVERSE, ROOT_MANIFEST, ALIAS_LEDGER)
    }

    pub(crate) fn validate_checked_in_coverage() -> Result<(), String> {
        Self::verified_alias_entries().map(|_| ())
    }

    pub(crate) fn validate_captured_bytes(bytes: &[u8]) -> Result<(), String> {
        if bytes != ALIAS_LEDGER.as_bytes() {
            return Err(
                "captured requirement source alias ledger differs from checker input".into(),
            );
        }
        Self::validate_coverage(REQUIREMENTS, SOURCE_UNIVERSE, ROOT_MANIFEST, ALIAS_LEDGER)
    }
}
