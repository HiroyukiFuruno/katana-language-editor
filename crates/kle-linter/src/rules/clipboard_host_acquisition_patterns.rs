use syn::spanned::Spanned;

const HOST_CRATES: &[&str] = &["katana", "katana_ui", "katana_core", "katana_platform"];
const MACOS_COMMANDS: &[&str] = &["osascript", "pbpaste", "pbcopy"];
const MIN_FILESYSTEM_SEGMENTS: usize = 3;

pub(super) fn use_tree_has_root(tree: &syn::UseTree, root: &str) -> bool {
    match tree {
        syn::UseTree::Path(path) => path.ident == root,
        syn::UseTree::Group(group) => group.items.iter().any(|item| use_tree_has_root(item, root)),
        syn::UseTree::Name(name) => name.ident == root,
        syn::UseTree::Rename(rename) => rename.ident == root,
        syn::UseTree::Glob(_) => false,
    }
}

pub(super) fn use_tree_has_host_crate(tree: &syn::UseTree) -> bool {
    HOST_CRATES
        .iter()
        .any(|crate_name| use_tree_has_root(tree, crate_name))
}

pub(super) fn path_has_root(path: &syn::Path, root: &str) -> bool {
    path.segments
        .first()
        .is_some_and(|segment| segment.ident == root)
}

pub(super) fn path_has_host_crate(path: &syn::Path) -> bool {
    HOST_CRATES
        .iter()
        .any(|crate_name| path_has_root(path, crate_name))
}

pub(super) fn is_filesystem_api(path: &syn::Path) -> bool {
    let segments: Vec<_> = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect();
    let is_fs = segments.len() >= MIN_FILESYSTEM_SEGMENTS
        && matches!(segments[0].as_str(), "std" | "tokio" | "async_std")
        && segments[1] == "fs";
    let is_file_type = segments
        .iter()
        .any(|name| matches!(name.as_str(), "File" | "OpenOptions"));
    let is_fs_operation = segments.last().is_some_and(|name| {
        matches!(name.as_str(), "read" | "read_to_string" | "write" | "copy")
            || (matches!(name.as_str(), "open" | "create" | "new") && is_file_type)
    });
    let is_image_operation = segments.first().is_some_and(|name| name == "image")
        && segments.last().is_some_and(|name| {
            matches!(
                name.as_str(),
                "open" | "save_buffer" | "save_buffer_with_format"
            )
        });
    is_fs && is_fs_operation || is_image_operation
}

pub(super) fn is_file_url_helper(name: &str) -> bool {
    matches!(
        name,
        "file_url_to_path" | "parse_file_url" | "decode_file_url_path" | "to_file_path"
    ) || ((name.contains("file_url") || name.contains("fileurl"))
        && (name.contains("path") || name.contains("parse") || name.contains("decode")))
}

pub(super) fn is_image_extension_helper(name: &str) -> bool {
    matches!(
        name,
        "supported_image_extension" | "path_has_image_extension"
    ) || (name.contains("image") && (name.contains("extension") || name.contains("supported")))
}

pub(super) fn is_macos_clipboard_command(call: &syn::ExprCall) -> bool {
    call.args.iter().any(is_macos_command_literal)
        && call
            .func
            .span()
            .source_text()
            .is_some_and(|text| text.contains("Command"))
}

pub(super) fn is_macos_clipboard_method_call(call: &syn::ExprMethodCall) -> bool {
    (call.method == "new" || call.method == "arg")
        && call.args.iter().any(is_macos_command_literal)
        && command_receiver_contains_command(&call.receiver)
}

fn command_receiver_contains_command(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::Path(path) => path
            .path
            .segments
            .iter()
            .any(|segment| segment.ident == "Command"),
        syn::Expr::MethodCall(call) => {
            call.method == "new" && command_receiver_contains_command(&call.receiver)
        }
        _ => false,
    }
}

fn is_macos_command_literal(expr: &syn::Expr) -> bool {
    matches!(expr, syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(value), ..
    }) if MACOS_COMMANDS.contains(&value.value().as_str()))
}

pub(super) fn is_file_url_parse_call(call: &syn::ExprCall) -> bool {
    let syn::Expr::Path(path) = call.func.as_ref() else {
        return false;
    };
    path.path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "parse")
        && path
            .path
            .segments
            .iter()
            .any(|segment| matches!(segment.ident.to_string().as_str(), "Url" | "Uri"))
        && call.args.iter().any(is_file_url_literal)
}

pub(super) fn is_file_url_literal(expr: &syn::Expr) -> bool {
    matches!(expr, syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(value), ..
    }) if value.value().starts_with("file://"))
}
