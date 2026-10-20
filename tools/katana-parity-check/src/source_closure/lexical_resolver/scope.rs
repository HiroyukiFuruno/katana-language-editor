use std::collections::BTreeMap;

use proc_macro2::Span;
use syn::{Item, ItemUse};

use super::super::ast_resolution::{LexicalImport, collect_lexical_imports};

#[derive(Clone, Debug)]
pub(super) struct AliasBinding {
    pub(super) source: Vec<String>,
    pub(super) span: Span,
}

#[derive(Clone, Debug, Default)]
pub(super) struct Scope {
    pub(super) aliases: BTreeMap<String, Vec<AliasBinding>>,
}

pub(super) fn scope_from_items(items: &[Item]) -> Scope {
    let mut scope = Scope::default();
    for item in items {
        let Item::Use(ItemUse { tree, .. }) = item else {
            continue;
        };
        let mut imports = Vec::new();
        collect_lexical_imports(tree, Vec::new(), &mut imports);
        for LexicalImport {
            source,
            local,
            glob,
            span,
        } in imports
        {
            if glob {
                continue;
            }
            if let Some(local) = local {
                scope
                    .aliases
                    .entry(local)
                    .or_default()
                    .push(AliasBinding { source, span });
            }
        }
    }
    scope
}
