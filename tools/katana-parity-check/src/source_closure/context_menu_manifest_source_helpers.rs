use super::super::super::fingerprint::sha256_hex;
#[path = "context_menu_manifest_authoring.rs"]
mod context_menu_manifest_authoring;
#[path = "context_menu_manifest_io_helpers.rs"]
mod context_menu_manifest_io_helpers;
use super::super::{CODE_BLOCK_SOURCE, ContextMenuLeaf, ROLE_LEAF};
use super::LocaleSourceInput;
use context_menu_manifest_io_helpers::{add_digest_leaf, locale_digests, snake_case};
use std::path::Path;

pub(super) fn path_digest(path: &str) -> String {
    context_menu_manifest_io_helpers::path_digest(path)
}

pub(super) fn span_digest(path: &str, source: &str, marker: &str) -> String {
    context_menu_manifest_io_helpers::span_digest(path, source, marker)
}

pub(super) fn read_utf8(root: &Path, relative: &str) -> Result<String, String> {
    context_menu_manifest_io_helpers::read_utf8(root, relative)
}

pub(super) fn read_locales(root: &Path) -> Result<Vec<LocaleSourceInput>, String> {
    context_menu_manifest_io_helpers::read_locales(root)
}

pub(super) fn derive_leaves(
    source: &str,
    code_block: &str,
    authoring_ops: &[String],
    kinds: &[String],
    locales: &[(String, Vec<u8>)],
) -> Result<Vec<ContextMenuLeaf>, String> {
    let mut leaves = Vec::new();
    add_leaf(
        &mut leaves,
        source,
        locales,
        "save",
        "action.save",
        "always",
    )?;
    add_leaf(
        &mut leaves,
        source,
        locales,
        "format",
        "action.format_markdown_file",
        "editable && extension_is_md_or_markdown",
    )?;
    for variant in authoring_ops {
        let name = snake_case(variant);
        let label_key = format!("search.command_author_{name}");
        add_leaf_with_label_key(
            &mut leaves,
            source,
            locales,
            &name,
            &format!("MarkdownAuthoringOp::{variant}"),
            "selection_or_structural",
            &label_key,
        )?;
    }
    for variant in kinds {
        let kind = snake_case(variant);
        add_code_leaf(
            &mut leaves,
            code_block,
            context_menu_manifest_io_helpers::code_block_label_digest(code_block, variant)?,
            variant,
            &format!("code_block/{kind}"),
            "always",
        )?;
    }
    add_leaf(
        &mut leaves,
        source,
        locales,
        "image_file",
        "command_ingest_image_file",
        "always",
    )?;
    add_leaf(
        &mut leaves,
        source,
        locales,
        "clipboard_image",
        "command_ingest_clipboard_image",
        "clipboard_image_available",
    )?;
    Ok(leaves)
}

fn add_leaf(
    leaves: &mut Vec<ContextMenuLeaf>,
    source: &str,
    locales: &[(String, Vec<u8>)],
    name: &str,
    source_marker: &str,
    condition: &str,
) -> Result<(), String> {
    if !source.contains(source_marker) {
        return Err(format!(
            "fixed context menu source is missing leaf marker {source_marker}"
        ));
    }
    let label_key = match name {
        "save" => "action.save",
        "format" => "action.format_markdown_file",
        "image_file" => "search.command_ingest_image_file",
        "clipboard_image" => "search.command_ingest_clipboard_image",
        key => {
            return add_leaf_with_label_key(
                leaves,
                source,
                locales,
                key,
                &format!("MarkdownAuthoringOp::{key}"),
                condition,
                &format!("search.command_author_{key}"),
            );
        }
    };
    add_leaf_with_label_key(
        leaves,
        source,
        locales,
        name,
        source_marker,
        condition,
        label_key,
    )
}

fn add_leaf_with_label_key(
    leaves: &mut Vec<ContextMenuLeaf>,
    source: &str,
    locales: &[(String, Vec<u8>)],
    name: &str,
    source_marker: &str,
    condition: &str,
    label_key: &str,
) -> Result<(), String> {
    if !source.contains(source_marker) {
        return Err(format!(
            "fixed context menu source is missing leaf marker {source_marker}"
        ));
    }
    let path = format!("editor.text_surface/context_menu/{name}");
    add_digest_leaf(
        leaves,
        source,
        &path,
        source_marker,
        condition,
        locale_digests(locales, label_key)?,
    )
}

fn add_code_leaf(
    leaves: &mut Vec<ContextMenuLeaf>,
    source: &str,
    label_digest: String,
    variant: &str,
    name: &str,
    condition: &str,
) -> Result<(), String> {
    let path = format!("editor.text_surface/context_menu/{name}");
    leaves.push(ContextMenuLeaf {
        path_digest: path_digest(&path),
        source_span_digest: span_digest(CODE_BLOCK_SOURCE, source, &format!("Self::{variant}")),
        role: ROLE_LEAF.into(),
        parent_path_digest: path_digest("editor.text_surface/context_menu"),
        enabled_condition_digest: sha256_hex(condition.as_bytes()),
        locale_label_digests: vec![label_digest],
    });
    Ok(())
}

pub(super) fn derive_direct_authoring_ops(source: &str) -> Result<Vec<String>, String> {
    context_menu_manifest_authoring::derive_direct_authoring_ops(source)
}
