use proc_macro2::Span;
use syn::{Attribute, UseGroup, UseName, UsePath, UseRename, UseTree, spanned::Spanned};

#[derive(Clone, Debug)]
pub(super) struct LexicalImport {
    pub(super) source: Vec<String>,
    pub(super) local: Option<String>,
    pub(super) glob: bool,
    pub(super) span: Span,
}

pub(super) fn collect_lexical_imports(
    tree: &UseTree,
    prefix: Vec<String>,
    out: &mut Vec<LexicalImport>,
) {
    match tree {
        UseTree::Path(UsePath { ident, tree, .. }) => {
            let mut next = prefix;
            next.push(ident.to_string());
            collect_lexical_imports(tree, next, out);
        }
        UseTree::Name(UseName { ident }) => {
            let mut source = prefix;
            source.push(ident.to_string());
            out.push(LexicalImport {
                source,
                local: Some(ident.to_string()),
                glob: false,
                span: tree.span(),
            });
        }
        UseTree::Rename(UseRename { ident, rename, .. }) => {
            let mut source = prefix;
            source.push(ident.to_string());
            out.push(LexicalImport {
                source,
                local: Some(rename.to_string()),
                glob: false,
                span: tree.span(),
            });
        }
        UseTree::Glob(_) => {
            let mut source = prefix;
            source.push("*".to_string());
            out.push(LexicalImport {
                source,
                local: None,
                glob: true,
                span: tree.span(),
            });
        }
        UseTree::Group(UseGroup { items, .. }) => {
            for item in items {
                collect_lexical_imports(item, prefix.clone(), out);
            }
        }
    }
}

pub(super) fn cfg_predicate(attr: &Attribute) -> Option<String> {
    if !attr.path().is_ident("cfg") {
        return None;
    }
    match &attr.meta {
        syn::Meta::List(list) => Some(list.tokens.to_string()),
        _ => None,
    }
}

pub(super) fn path_to_symbol(segments: &[String]) -> String {
    segments.join("::")
}

pub(super) fn span_to_text(file: &str, span: &Span) -> String {
    let start = span.start();
    let end = span.end();
    format!(
        "katana:{file}:{}:{}-{}:{}",
        start.line, start.column, end.line, end.column
    )
}
