use syn::ItemMod;
use syn::spanned::Spanned;

use super::SourceClosureVisitor;

impl<'a> SourceClosureVisitor<'a> {
    pub(crate) fn visit_mod(&mut self, node: &ItemMod) {
        if node.content.is_none() {
            let to_path = self
                .resolve_module(&node.ident.to_string())
                .and_then(|candidate| candidate.canonicalize().ok());
            self.record_mod_edge(to_path, &node.ident.to_string(), node.span());
        }
    }
}
