use std::path::Path;

use serde::{Deserialize, Serialize};

use super::root_model::ManifestRoot;

#[path = "context_menu_manifest_generation.rs"]
mod context_menu_manifest_generation;
#[path = "context_menu_manifest_validation.rs"]
mod context_menu_manifest_validation;

use context_menu_manifest_validation::validate_manifest;

const CONTEXT_MENU_SOURCE: &str = "crates/katana-ui/src/views/panels/editor/context_menu.rs";
const CODE_BLOCK_SOURCE: &str = "crates/katana-ui/src/markdown_authoring_op.rs";
const IMAGE_INGEST_SOURCE: &str =
    "crates/katana-ui/src/views/panels/editor/context_menu_image_ingest.rs";
const LOCALES_DIRECTORY: &str = "crates/katana-ui/locales";
const LOCALES_METADATA_FILE: &str = "languages.json";
const GENERATED_BY: &str = "katana-source-closure-context-menu-generator";
const ROLE_EDITOR: &str = "MultilineTextInput";
const ROLE_MENU: &str = "Menu";
const ROLE_LEAF: &str = "Button";
const DIRECT_AUTHORING_COUNT: usize = 13;
const CODE_BLOCK_KIND_COUNT: usize = 17;
const NON_AUTHORING_LEAF_COUNT: usize = 4;
const SHA256_HEX_LENGTH: usize = 64;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ContextMenuTargetManifest {
    pub(crate) schema_version: String,
    pub(crate) generated_by: String,
    pub(crate) katana_revision: String,
    pub(crate) profile_fingerprint: String,
    pub(crate) source_closure_fingerprint: String,
    pub(crate) surface: ContextMenuSurface,
    pub(crate) menu: ContextMenuMenu,
    pub(crate) leaves: Vec<ContextMenuLeaf>,
    pub(crate) routes: Vec<ContextMenuRoute>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ContextMenuSurface {
    pub(crate) source_span_digest: String,
    pub(crate) role: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ContextMenuMenu {
    pub(crate) source_span_digest: String,
    pub(crate) role: String,
    pub(crate) path_digest: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ContextMenuLeaf {
    pub(crate) path_digest: String,
    pub(crate) source_span_digest: String,
    pub(crate) role: String,
    pub(crate) parent_path_digest: String,
    pub(crate) enabled_condition_digest: String,
    pub(crate) locale_label_digests: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum ContextMenuRoute {
    SecondaryPointer,
    ShiftF10,
    AccesskitInvoke,
}

pub(crate) struct ContextMenuManifestGenerator;

impl ContextMenuManifestGenerator {
    pub(crate) fn generate(
        root: &ManifestRoot,
        katana_root: &Path,
    ) -> Result<ContextMenuTargetManifest, String> {
        context_menu_manifest_generation::generate(root, katana_root)
    }
}

pub(super) fn validate_manifest_contract(
    manifest: &ContextMenuTargetManifest,
    root: &ManifestRoot,
) -> Result<(), String> {
    validate_manifest(manifest, root)
}

#[cfg(test)]
mod tests {
    use super::super::fixed_reference::fixed_reference_root;
    use super::super::operational_input::FIXED_KATANA_REVISION;
    use super::*;
    use context_menu_manifest_generation::{generate_from_sources, read_locales, read_utf8};

    type FixedInputs = (ManifestRoot, String, String, Vec<(String, Vec<u8>)>);

    fn root() -> ManifestRoot {
        ManifestRoot {
            schema_version: "1".into(),
            katana_revision: FIXED_KATANA_REVISION.into(),
            katana_tree_fingerprint: "sha256:tree".into(),
            katana_external_ui_fingerprint: "sha256:external".into(),
            user_mandated_extensions_fingerprint: "sha256:user".into(),
            source_universe_fingerprint: "sha256:source-universe".into(),
            requirement_source_aliases_fingerprint: "sha256:aliases".into(),
            kle_tree_fingerprint: "sha256:kle".into(),
            kuc_tree_fingerprint: "sha256:kuc".into(),
            release_profile_matrix_fingerprint: "sha256:profile".into(),
            generator_fingerprint: "sha256:generator".into(),
            generated_at_utc: "1970-01-01T00:00:00Z".into(),
            static_leaf_count: None,
            expected_leaf_ids: None,
        }
    }

    fn fixed_inputs() -> Result<FixedInputs, String> {
        let root = root();
        let fixed = fixed_reference_root()?;
        let context = format!(
            "{}\n{}",
            read_utf8(fixed, CONTEXT_MENU_SOURCE)?,
            read_utf8(fixed, IMAGE_INGEST_SOURCE)?
        );
        let code = read_utf8(fixed, CODE_BLOCK_SOURCE)?;
        let locales = read_locales(fixed)?;
        Ok((root, context, code, locales))
    }

    #[test]
    fn fixed_source_generates_complete_dynamic_inventory() -> Result<(), String> {
        let (root, context, code, locales) = fixed_inputs()?;
        let manifest = generate_from_sources(&root, &context, &code, &locales)?;
        assert_eq!(
            manifest.leaves.len(),
            NON_AUTHORING_LEAF_COUNT + DIRECT_AUTHORING_COUNT + CODE_BLOCK_KIND_COUNT
        );
        assert_eq!(manifest.routes.len(), 3);
        Ok(())
    }

    #[test]
    fn authoring_order_is_source_derived_and_stale_inventory_fails_closed() -> Result<(), String> {
        let (root, context, code, locales) = fixed_inputs()?;
        let baseline = generate_from_sources(&root, &context, &code, &locales)?;
        let omitted = context.replacen("MarkdownAuthoringOp::InsertTable", "", 1);
        assert!(generate_from_sources(&root, &omitted, &code, &locales).is_err());
        let added = format!(
            "{context}\nSelf::author_button(ui, action, label, MarkdownAuthoringOp::NewOperation, true);"
        );
        assert!(generate_from_sources(&root, &added, &code, &locales).is_err());
        assert!(!baseline.leaves.is_empty());
        Ok(())
    }

    #[test]
    fn code_kind_order_is_source_derived_and_stale_inventory_fails_closed() -> Result<(), String> {
        let (root, context, code, locales) = fixed_inputs()?;
        let omitted = code.replacen("Self::Sql,", "", 1);
        assert!(generate_from_sources(&root, &context, &omitted, &locales).is_err());
        let duplicated = code.replacen("Self::Sql,", "Self::Text,", 1);
        assert!(generate_from_sources(&root, &context, &duplicated, &locales).is_err());
        Ok(())
    }

    #[test]
    fn manifest_validation_rejects_mutation_and_omission() -> Result<(), String> {
        let (root, context, code, locales) = fixed_inputs()?;
        let mut manifest = generate_from_sources(&root, &context, &code, &locales)?;
        manifest.leaves.pop();
        assert!(validate_manifest_contract(&manifest, &root).is_err());
        Ok(())
    }

    #[test]
    fn serialized_manifest_contains_no_raw_labels_or_actions() -> Result<(), String> {
        let (root, context, code, locales) = fixed_inputs()?;
        let manifest = generate_from_sources(&root, &context, &code, &locales)?;
        let json = serde_json::to_string(&manifest)
            .map_err(|error| format!("manifest JSON serialization failed: {error}"))?;
        assert!(!json.contains("EditorAction"));
        assert!(!json.contains("AppAction"));
        assert!(!json.contains("Save"));
        Ok(())
    }
}
