use std::collections::BTreeSet;

use super::super::fixed_reference::fixed_reference_root;
use super::super::operational_input::FIXED_KATANA_REVISION;
use super::super::root_model::ManifestRoot;
use super::context_menu_manifest_generation::{generate_from_sources, read_locales, read_utf8};
use super::{
    CODE_BLOCK_KIND_COUNT, CODE_BLOCK_SOURCE, CONTEXT_MENU_SOURCE, ContextMenuRoute,
    ContextMenuTargetManifest, DIRECT_AUTHORING_COUNT, GENERATED_BY, IMAGE_INGEST_SOURCE,
    NON_AUTHORING_LEAF_COUNT, ROLE_EDITOR, ROLE_LEAF, ROLE_MENU, SHA256_HEX_LENGTH,
};

pub(super) fn validate_manifest(
    manifest: &ContextMenuTargetManifest,
    root: &ManifestRoot,
) -> Result<(), String> {
    if manifest.katana_revision != FIXED_KATANA_REVISION
        || manifest.profile_fingerprint != root.release_profile_matrix_fingerprint
        || manifest.source_closure_fingerprint != root.katana_tree_fingerprint
    {
        return Err(
            "context menu manifest root fingerprints do not match fixed source closure".into(),
        );
    }
    validate_manifest_shape(manifest)?;
    let fixed_root = fixed_reference_root()?;
    let context = read_utf8(fixed_root, CONTEXT_MENU_SOURCE)?;
    let code_block = read_utf8(fixed_root, CODE_BLOCK_SOURCE)?;
    let image_ingest = read_utf8(fixed_root, IMAGE_INGEST_SOURCE)?;
    let locales = read_locales(fixed_root)?;
    let expected = generate_from_sources(
        root,
        &format!("{context}\n{image_ingest}"),
        &code_block,
        &locales,
    )?;
    if manifest.surface != expected.surface
        || manifest.menu != expected.menu
        || manifest.leaves != expected.leaves
        || manifest.routes != expected.routes
    {
        return Err("context menu manifest does not match the fixed source closure".into());
    }
    Ok(())
}

pub(super) fn validate_manifest_shape(manifest: &ContextMenuTargetManifest) -> Result<(), String> {
    if manifest.schema_version != "1" || manifest.generated_by != GENERATED_BY {
        return Err("context menu manifest schema or generator identity is invalid".into());
    }
    if !is_digest(&manifest.surface.source_span_digest)
        || !is_digest(&manifest.menu.source_span_digest)
        || !is_digest(&manifest.menu.path_digest)
        || manifest.surface.role != ROLE_EDITOR
        || manifest.menu.role != ROLE_MENU
    {
        return Err("context menu manifest surface or menu evidence is invalid".into());
    }
    if manifest.routes
        != [
            ContextMenuRoute::SecondaryPointer,
            ContextMenuRoute::ShiftF10,
            ContextMenuRoute::AccesskitInvoke,
        ]
    {
        return Err("context menu manifest routes are not the supported three routes".into());
    }
    if manifest.leaves.len()
        != NON_AUTHORING_LEAF_COUNT + DIRECT_AUTHORING_COUNT + CODE_BLOCK_KIND_COUNT
    {
        return Err("context menu manifest leaf inventory is incomplete".into());
    }
    let mut paths = BTreeSet::new();
    let mut spans = BTreeSet::new();
    for leaf in &manifest.leaves {
        let duplicate_path = !paths.insert(leaf.path_digest.clone());
        let duplicate_span = !spans.insert(leaf.source_span_digest.clone());
        if leaf.path_digest.is_empty()
            || duplicate_path
            || duplicate_span
            || !is_digest(&leaf.source_span_digest)
            || !is_digest(&leaf.enabled_condition_digest)
            || leaf.role != ROLE_LEAF
            || leaf.parent_path_digest != manifest.menu.path_digest
            || leaf
                .locale_label_digests
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            || leaf
                .locale_label_digests
                .iter()
                .any(|digest| !is_digest(digest))
        {
            return Err("context menu manifest contains duplicate or invalid leaf evidence".into());
        }
    }
    Ok(())
}

fn is_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == SHA256_HEX_LENGTH && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
    })
}
