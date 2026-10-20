use std::path::Path;

pub(super) const RULE: &str = "status-diagnostics-ownership-boundary";
pub(super) const MESSAGE: &str = "KLE and Storybook may forward only the opaque KUC projection, generic scenario identifiers, and closed receipts; StatusBar and DiagnosticsList internals belong to KUC.";

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
    let name = normalize(name);
    name.contains("statusbar")
        || name.contains("diagnosticslist")
        || name.starts_with("statusdiagnosticsprojectionlease")
}

fn is_allowed_symbol(name: &str) -> bool {
    matches!(
        normalize(name).as_str(),
        "eguitextcommandsurfacehostprojectionlease"
            | "fulltextcommandsurfacescenarioid"
            | "workspacetabs"
            | "eguitextcommandsurfacehostrooteventdispatchreceipt"
            | "eguitextcommandsurfacehostrooteventforwardingreceipt"
            | "opaquerootartifactreceipt"
            | "kucrootbindingreceipt"
    )
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
