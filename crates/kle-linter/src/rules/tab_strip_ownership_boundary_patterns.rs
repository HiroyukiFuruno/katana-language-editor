use std::path::Path;

pub(super) const RULE: &str = "tab-strip-ownership-boundary";
pub(super) const MESSAGE: &str = "KLE and Storybook may forward only the opaque KUC projection, generic scenario identifiers, and closed receipts; TabStrip internals belong to KUC.";

pub(super) fn is_target(path: &Path) -> bool {
    let path = path.to_string_lossy();
    path.contains("crates/katana-language-editor/src/")
        || path.contains("crates/katana-language-editor-egui/src/")
        || path.contains("tools/kle-storybook/src/")
}

pub(super) fn is_test_source(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str().to_string_lossy() == "tests")
        || path.file_name().is_some_and(|name| {
            let name = name.to_string_lossy();
            name == "tests.rs" || name.ends_with("_tests.rs")
        })
}

pub(super) fn is_cfg_test_module(attributes: &[syn::Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute.path().is_ident("cfg")
            && matches!(&attribute.meta, syn::Meta::List(list) if list.tokens.to_string().contains("test"))
    })
}

pub(super) fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

pub(super) fn is_forbidden_symbol(name: &str) -> bool {
    if is_allowed_symbol(name) {
        return false;
    }
    name.starts_with("closeabletab")
        || name.starts_with("sanitizedtab")
        || name.starts_with("workspacetab")
        || name.starts_with("workspacegroup")
        || name.contains("tabstrip")
        || is_tab_or_group_identity(name)
        || is_raw_group_name(name)
        || is_tab_transport(name)
        || is_tab_palette(name)
        || is_tab_render_symbol(name)
}

pub(super) fn is_tab_context_name(name: &str) -> bool {
    name.contains("tab") || name.contains("group")
}

pub(super) fn is_direct_egui_tab_child(name: &str) -> bool {
    matches!(
        name,
        "ui" | "button" | "textedit" | "image" | "imagebutton" | "richtext"
    )
}

pub(super) fn is_direct_egui_child_method(name: &str) -> bool {
    matches!(
        name,
        "add"
            | "button"
            | "coloredlabel"
            | "horizontal"
            | "label"
            | "selectablevalue"
            | "separator"
            | "texteditsingleline"
            | "texteditmultiline"
            | "vertical"
    )
}

pub(super) fn is_raw_color_hex(value: &str) -> bool {
    let Some(hex) = value.strip_prefix('#') else {
        return false;
    };
    matches!(hex.len(), 3 | 4 | 6 | 8) && hex.chars().all(|character| character.is_ascii_hexdigit())
}

pub(super) fn visit_use_tree(tree: &syn::UseTree, visit_identifier: &mut impl FnMut(&syn::Ident)) {
    match tree {
        syn::UseTree::Path(path) => visit_use_tree(&path.tree, visit_identifier),
        syn::UseTree::Name(name) => visit_identifier(&name.ident),
        syn::UseTree::Rename(rename) => {
            visit_identifier(&rename.ident);
            visit_identifier(&rename.rename);
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                visit_use_tree(item, visit_identifier);
            }
        }
        syn::UseTree::Glob(_) => {}
    }
}

fn is_allowed_symbol(name: &str) -> bool {
    matches!(
        name,
        "eguitextcommandsurfacehostprojectionlease"
            | "fulltextcommandsurfacescenarioid"
            | "workspacetabs"
            | "eguitextcommandsurfacehostrooteventdispatchreceipt"
            | "eguitextcommandsurfacehostrooteventforwardingreceipt"
            | "opaquerootartifactreceipt"
            | "kucrootbindingreceipt"
    )
}

fn is_tab_or_group_identity(name: &str) -> bool {
    (name.contains("tab") || name.contains("group"))
        && ["id", "ids", "index", "indices", "path", "paths"]
            .iter()
            .any(|suffix| name.ends_with(suffix))
}

fn is_raw_group_name(name: &str) -> bool {
    name.contains("group")
        && ["name", "title", "label", "input", "query", "state"]
            .iter()
            .any(|part| name.contains(part))
}

fn is_tab_transport(name: &str) -> bool {
    (name.contains("tab") || name.contains("group"))
        && [
            "port", "event", "adapter", "action", "route", "target", "dispatch",
        ]
        .iter()
        .any(|part| name.contains(part))
}

fn is_tab_palette(name: &str) -> bool {
    (name.contains("tab") || name.contains("group"))
        && ["color", "colors", "palette", "theme"]
            .iter()
            .any(|part| name.contains(part))
}

fn is_tab_render_symbol(name: &str) -> bool {
    name.contains("tab")
        && ["render", "draw", "show", "paint", "child", "ui"]
            .iter()
            .any(|part| name.contains(part))
}
