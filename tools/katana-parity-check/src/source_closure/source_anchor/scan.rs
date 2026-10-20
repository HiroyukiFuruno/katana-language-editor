use proc_macro2::Span;
use syn::{File, Item, Macro, spanned::Spanned, visit::Visit};

use super::super::ast_scan::{VecMacroInput, macro_recovery};
use super::model::{AnchorFact, AnchorKind, ScanFacts, UnresolvedSyntax};
use super::spans::{make_fact, make_unresolved};

pub(super) fn scan_file(source: &str, syntax: &File) -> Result<ScanFacts, String> {
    let mut scanner = AnchorScanner {
        source,
        owner: Vec::new(),
        anchors: Vec::new(),
        unresolved: Vec::new(),
        span_error: None,
        bare_vec_ambiguous: VecMacroInput::file_has_bare_vec_ambiguity(syntax),
    };
    scanner.visit_file(syntax);
    if let Some(error) = scanner.span_error {
        return Err(error);
    }
    Ok((scanner.anchors, scanner.unresolved))
}

pub(super) struct AnchorScanner<'a> {
    source: &'a str,
    owner: Vec<String>,
    anchors: Vec<AnchorFact>,
    unresolved: Vec<UnresolvedSyntax>,
    span_error: Option<String>,
    bare_vec_ambiguous: bool,
}

impl AnchorScanner<'_> {
    fn push_anchor(&mut self, kind: AnchorKind, name: String, span: Span) {
        match make_fact(self.source, &self.owner, kind, name, span) {
            Ok(fact) => self.anchors.push(fact),
            Err(error) => self.span_error = Some(error),
        }
    }

    fn with_owner<T>(&mut self, name: String, visit: impl FnOnce(&mut Self) -> T) -> T {
        self.owner.push(name);
        let result = visit(self);
        self.owner.pop();
        result
    }
}

impl<'ast> Visit<'ast> for AnchorScanner<'_> {
    fn visit_item(&mut self, item: &'ast Item) {
        match item {
            Item::Enum(node) => self.with_owner(node.ident.to_string(), |this| {
                for variant in &node.variants {
                    this.push_anchor(
                        AnchorKind::EnumVariant,
                        variant.ident.to_string(),
                        variant.span(),
                    );
                    syn::visit::visit_variant(this, variant);
                }
            }),
            Item::Fn(node) => {
                let name = node.sig.ident.to_string();
                let kind = if node.attrs.iter().any(|attr| {
                    attr.path()
                        .segments
                        .last()
                        .is_some_and(|part| part.ident == "test")
                }) {
                    AnchorKind::TestDeclaration
                } else {
                    AnchorKind::FunctionDeclaration
                };
                self.push_anchor(kind, name.clone(), node.span());
                self.with_owner(name, |this| syn::visit::visit_item_fn(this, node));
            }
            Item::Impl(node) => self.with_owner(impl_owner(&node.self_ty), |this| {
                if !node.generics.params.is_empty() {
                    match make_unresolved(
                        this.source,
                        &this.owner,
                        node.span(),
                        "generic impl owner is syntax-only; semantic type identity is unresolved"
                            .into(),
                    ) {
                        Ok(record) => this.unresolved.push(record),
                        Err(error) => this.span_error = Some(error),
                    }
                }
                syn::visit::visit_item_impl(this, node)
            }),
            Item::Trait(node) => self.with_owner(node.ident.to_string(), |this| {
                syn::visit::visit_item_trait(this, node)
            }),
            Item::Mod(node) => self.with_owner(node.ident.to_string(), |this| {
                syn::visit::visit_item_mod(this, node)
            }),
            _ => syn::visit::visit_item(self, item),
        }
    }

    fn visit_expr_struct(&mut self, node: &'ast syn::ExprStruct) {
        let name = node
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        self.push_anchor(AnchorKind::StructConstruction, name, node.span());
        syn::visit::visit_expr_struct(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let name = node.sig.ident.to_string();
        let kind = if node.attrs.iter().any(|attr| {
            attr.path()
                .segments
                .last()
                .is_some_and(|part| part.ident == "test")
        }) {
            AnchorKind::TestDeclaration
        } else {
            AnchorKind::FunctionDeclaration
        };
        self.push_anchor(kind, name.clone(), node.span());
        self.with_owner(name, |this| syn::visit::visit_impl_item_fn(this, node));
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        let name = node.sig.ident.to_string();
        self.push_anchor(AnchorKind::FunctionDeclaration, name.clone(), node.span());
        self.with_owner(name, |this| syn::visit::visit_trait_item_fn(this, node));
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        let span = node.span();
        let result = macro_recovery::VecMacroInput::parse_macro(node, self.bare_vec_ambiguous);
        let reason = match result {
            Ok(VecMacroInput::List(items)) => {
                for item in items {
                    self.visit_expr(&item);
                }
                "vec macro tokens recovered as syntax only; expansion and callback behavior remain unresolved".to_owned()
            }
            Ok(VecMacroInput::Repeat { value, count }) => {
                self.visit_expr(&value);
                self.visit_expr(&count);
                "vec repeat tokens recovered once as syntax only; expansion and evaluation remain unresolved".to_owned()
            }
            Err(reason) => reason,
        };
        match make_unresolved(self.source, &self.owner, span, reason) {
            Ok(record) => self.unresolved.push(record),
            Err(error) => self.span_error = Some(error),
        }
    }
}

fn impl_owner(self_type: &syn::Type) -> String {
    match self_type {
        syn::Type::Path(node) => format!(
            "impl {}",
            node.path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>()
                .join("::")
        ),
        _ => "impl <non-path type syntax>".into(),
    }
}
