use syn::parse::{Parse, ParseStream};
use syn::visit::{self, Visit};
use syn::{Expr, File, ItemMacro, ItemUse, Macro, Token, spanned::Spanned};

use super::super::ast_resolution::collect_lexical_imports;
use super::super::ast_scan::SourceClosureVisitor;
use super::super::scan_state::InputOriginCandidate;

enum VecMacroInput {
    List(Vec<Expr>),
    Repeat { value: Box<Expr>, count: Box<Expr> },
}

impl Parse for VecMacroInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::List(Vec::new()));
        }
        let first = input.parse::<Expr>()?;
        if input.peek(Token![;]) {
            input.parse::<Token![;]>()?;
            let count = input.parse::<Expr>()?;
            if !input.is_empty() {
                return Err(input.error("unexpected tokens after vec repeat expression"));
            }
            return Ok(Self::Repeat {
                value: Box::new(first),
                count: Box::new(count),
            });
        }
        let mut items = vec![first];
        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            items.push(input.parse::<Expr>()?);
        }
        if !input.is_empty() {
            return Err(input.error("unexpected tokens after vec list expression"));
        }
        Ok(Self::List(items))
    }
}

pub(super) fn file_has_bare_vec_ambiguity(file: &File) -> bool {
    let mut visitor = VecNameAmbiguityVisitor { ambiguous: false };
    visitor.visit_file(file);
    visitor.ambiguous
}

struct VecNameAmbiguityVisitor {
    ambiguous: bool,
}

impl<'ast> Visit<'ast> for VecNameAmbiguityVisitor {
    fn visit_item_macro(&mut self, node: &'ast ItemMacro) {
        if node.ident.as_ref().is_some_and(|ident| ident == "vec") {
            self.ambiguous = true;
        }
        visit::visit_item_macro(self, node);
    }

    fn visit_item_use(&mut self, node: &'ast ItemUse) {
        let mut imports = Vec::new();
        collect_lexical_imports(&node.tree, Vec::new(), &mut imports);
        self.ambiguous |= imports
            .iter()
            .any(|import| import.glob || import.local.as_deref() == Some("vec"));
        visit::visit_item_use(self, node);
    }
}

fn is_supported_vec_path(path: &syn::Path, bare_name_ambiguous: bool) -> bool {
    let names = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    match (path.leading_colon.is_some(), names.as_slice()) {
        (false, [name]) if name == "vec" => !bare_name_ambiguous,
        (true, [root, name]) if name == "vec" => root == "std" || root == "alloc",
        _ => false,
    }
}

fn parse_vec_macro(node: &Macro, bare_name_ambiguous: bool) -> Result<VecMacroInput, String> {
    if !is_supported_vec_path(&node.path, bare_name_ambiguous) {
        return Err("macro path is not a supported coherent vec factory".to_string());
    }
    syn::parse2::<VecMacroInput>(node.tokens.clone())
        .map_err(|error| format!("vec macro tokens are malformed and remain unresolved: {error}"))
}

impl<'a> SourceClosureVisitor<'a> {
    pub(crate) fn visit_recovered_macro(&mut self, node: &Macro) -> String {
        let boundary = InputOriginCandidate {
        kind: "macro_token_boundary".to_string(),
        condition_syntax: "macro_token_boundary".to_string(),
        condition_span: super::super::ast_resolution::span_to_text(
            self.current_relative_path,
            &node.span(),
        ),
        boundary_syntax: "macro_token_boundary".to_string(),
        boundary_span: super::super::ast_resolution::span_to_text(
            self.current_relative_path,
            &node.span(),
        ),
        unresolved_reason: Some(
            "macro token recovery is syntax-only and does not prove expansion, input provenance, or host effect".to_string(),
        ),
    };
        match parse_vec_macro(node, self.macro_recovery_ambiguous) {
            Ok(VecMacroInput::List(items)) => {
                self.with_input_candidate(boundary, |this| {
                    for item in items {
                        this.visit_expr(&item);
                    }
                });
                "supported vec macro recovered; syntax-only recovery does not prove expansion, input provenance, or host effect".to_string()
            }
            Ok(VecMacroInput::Repeat { value, count }) => {
                self.with_input_candidate(boundary, |this| {
                    this.visit_expr(value.as_ref());
                    this.visit_expr(count.as_ref());
                });
                "supported vec repeat macro recovered once; syntax-only recovery does not prove expansion, input provenance, or host effect".to_string()
            }
            Err(reason) => reason,
        }
    }
}

#[cfg(test)]
#[path = "macro_recovery_tests.rs"]
mod macro_recovery_tests;

#[cfg(test)]
#[path = "macro_recovery_fixed_source_tests.rs"]
mod macro_recovery_fixed_source_tests;

#[cfg(test)]
#[path = "macro_recovery_span_tests.rs"]
mod macro_recovery_span_tests;
