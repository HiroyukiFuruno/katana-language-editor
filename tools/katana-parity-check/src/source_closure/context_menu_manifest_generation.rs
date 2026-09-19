use std::collections::BTreeSet;
use std::path::Path;

use super::super::operational_input::FIXED_KATANA_REVISION;
use super::super::root_model::ManifestRoot;
#[path = "context_menu_manifest_source_helpers.rs"]
mod context_menu_manifest_source_helpers;

use super::{
    CODE_BLOCK_KIND_COUNT, CODE_BLOCK_SOURCE, CONTEXT_MENU_SOURCE, ContextMenuMenu,
    ContextMenuRoute, ContextMenuSurface, ContextMenuTargetManifest, GENERATED_BY,
    IMAGE_INGEST_SOURCE, NON_AUTHORING_LEAF_COUNT, ROLE_EDITOR, ROLE_MENU,
};

use context_menu_manifest_source_helpers::{derive_direct_authoring_ops, path_digest, span_digest};

pub(super) fn read_utf8(root: &Path, relative: &str) -> Result<String, String> {
    context_menu_manifest_source_helpers::read_utf8(root, relative)
}

pub(super) fn read_locales(root: &Path) -> Result<Vec<LocaleSourceInput>, String> {
    context_menu_manifest_source_helpers::read_locales(root)
}

pub(super) type LocaleSourceInput = (String, Vec<u8>);

pub(super) fn generate(
    root: &ManifestRoot,
    katana_root: &Path,
) -> Result<ContextMenuTargetManifest, String> {
    let context = read_utf8(katana_root, CONTEXT_MENU_SOURCE)?;
    let code_block = read_utf8(katana_root, CODE_BLOCK_SOURCE)?;
    let image_ingest = read_utf8(katana_root, IMAGE_INGEST_SOURCE)?;
    let locales = read_locales(katana_root)?;
    let context_with_image_ingest = format!("{context}\n{image_ingest}");
    generate_from_sources(root, &context_with_image_ingest, &code_block, &locales)
}

pub(super) fn generate_from_sources(
    root: &ManifestRoot,
    context: &str,
    code_block: &str,
    locales: &[(String, Vec<u8>)],
) -> Result<ContextMenuTargetManifest, String> {
    if root.katana_revision != FIXED_KATANA_REVISION {
        return Err("context menu manifest requires the fixed KatanA revision".into());
    }
    if context.is_empty() || code_block.is_empty() || locales.is_empty() {
        return Err("context menu source closure inputs are incomplete".into());
    }
    require_context_markers(context)?;
    let authoring_ops = derive_direct_authoring_ops(context)?;
    let kinds = derive_code_block_kinds(code_block)?;
    let leaves = context_menu_manifest_source_helpers::derive_leaves(
        context,
        code_block,
        &authoring_ops,
        &kinds,
        locales,
    )?;
    let expected_leaf_count = NON_AUTHORING_LEAF_COUNT + authoring_ops.len() + kinds.len();
    if leaves.len() != expected_leaf_count {
        return Err(format!(
            "fixed context menu inventory must contain {expected_leaf_count} leaves, got {}",
            leaves.len()
        ));
    }
    let manifest = ContextMenuTargetManifest {
        schema_version: "1".into(),
        generated_by: GENERATED_BY.into(),
        katana_revision: root.katana_revision.clone(),
        profile_fingerprint: root.release_profile_matrix_fingerprint.clone(),
        source_closure_fingerprint: root.katana_tree_fingerprint.clone(),
        surface: ContextMenuSurface {
            source_span_digest: span_digest(CONTEXT_MENU_SOURCE, context, "response.context_menu"),
            role: ROLE_EDITOR.into(),
        },
        menu: ContextMenuMenu {
            source_span_digest: span_digest(CONTEXT_MENU_SOURCE, context, "response.context_menu"),
            role: ROLE_MENU.into(),
            path_digest: path_digest("editor.text_surface/context_menu"),
        },
        leaves,
        routes: vec![
            ContextMenuRoute::SecondaryPointer,
            ContextMenuRoute::ShiftF10,
            ContextMenuRoute::AccesskitInvoke,
        ],
    };
    super::context_menu_manifest_validation::validate_manifest_shape(&manifest)?;
    Ok(manifest)
}

fn require_context_markers(source: &str) -> Result<(), String> {
    for marker in [
        "response.context_menu",
        "SaveDocument",
        "FormatMarkdownFile",
        "context_menu_image_ingest",
    ] {
        if !source.contains(marker) {
            return Err(format!(
                "fixed context menu source is missing marker {marker}"
            ));
        }
    }
    Ok(())
}

fn derive_code_block_kinds(source: &str) -> Result<Vec<String>, String> {
    let start = source
        .find("const ALL:")
        .ok_or_else(|| "CodeBlockKind::ALL source is missing".to_string())?;
    let end = source[start..]
        .find("];\n\n    pub fn all")
        .map(|offset| start + offset)
        .ok_or_else(|| "CodeBlockKind::ALL source terminator is missing".to_string())?;
    let block = &source[start..end];
    let mut kinds = Vec::new();
    let mut seen = BTreeSet::new();
    for line in block.lines() {
        let Some(kind) = line.trim().strip_prefix("Self::") else {
            continue;
        };
        let kind = kind.trim_end_matches(',');
        if !kind.is_empty() {
            if !seen.insert(kind.to_string()) {
                return Err(format!(
                    "fixed CodeBlockKind::all inventory contains duplicate variant {kind}"
                ));
            }
            kinds.push(kind.to_string());
        }
    }
    if kinds.len() != CODE_BLOCK_KIND_COUNT {
        return Err(format!(
            "fixed CodeBlockKind::all inventory is not exactly {CODE_BLOCK_KIND_COUNT} unique entries"
        ));
    }
    Ok(kinds)
}
