use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{FnArg, ItemImpl, ItemTrait, Receiver, ReceiverKind, Type};

use super::super::ast_resolution::span_to_text;
use super::super::scan_state::{FreeFunctionDefinition, HandlerBodyIndex, MethodDefinition};
use super::SourceClosureVisitor;

impl<'a> SourceClosureVisitor<'a> {
    pub(crate) fn record_inherent_methods(&mut self, node: &ItemImpl) {
        let inherent = node.trait_.is_none();
        let implementation = format!(
            "{}{}",
            if inherent { "inherent:" } else { "trait_impl:" },
            Self::type_shape(&node.self_ty)
        );
        for item in &node.items {
            let syn::ImplItem::Fn(function) = item else {
                continue;
            };
            self.record_method_definition(
                (&function.sig.ident.to_string(), &function.sig.inputs),
                implementation.clone(),
                inherent,
                Self::type_shape(&node.self_ty),
                (function.sig.ident.span(), function.span()),
                super::super::ast_visit::index_handler_body(
                    &function.block,
                    &function.attrs,
                    self.current_relative_path,
                ),
            );
        }
    }

    pub(crate) fn record_free_function(&mut self, node: &syn::ItemFn) {
        self.state
            .record_free_function_definition(FreeFunctionDefinition {
                file: self.current_relative_path.to_string(),
                symbol: self.current_symbol(),
                function: node.sig.ident.to_string(),
                function_span: span_to_text(self.current_relative_path, &node.sig.ident.span()),
                source_span: span_to_text(self.current_relative_path, &node.span()),
                direct_path_calls: super::super::ast_visit::direct_path_calls(&node.block),
            });
    }

    pub(crate) fn record_trait_methods(&mut self, node: &ItemTrait) {
        for item in &node.items {
            let syn::TraitItem::Fn(function) = item else {
                continue;
            };
            self.record_method_definition(
                (&function.sig.ident.to_string(), &function.sig.inputs),
                format!("trait:{}", node.ident),
                false,
                format!("trait:{}", node.ident),
                (function.sig.ident.span(), function.span()),
                (
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    vec!["trait method body is not uniquely linked".to_string()],
                ),
            );
        }
    }

    fn record_method_definition(
        &mut self,
        signature: (&str, &syn::punctuated::Punctuated<FnArg, syn::Token![,]>),
        implementation: String,
        inherent: bool,
        receiver_type: String,
        spans: (Span, Span),
        body: HandlerBodyIndex,
    ) {
        let (method, inputs) = signature;
        let receiver_shape = inputs
            .first()
            .and_then(|input| match input {
                FnArg::Receiver(receiver) => Some(receiver_shape(receiver)),
                _ => None,
            })
            .unwrap_or_else(|| "static".to_string());
        self.state.record_method_definition(MethodDefinition {
            file: self.current_relative_path.to_string(),
            implementation,
            method: method.to_string(),
            receiver_shape,
            method_span: span_to_text(self.current_relative_path, &spans.0),
            source_span: span_to_text(self.current_relative_path, &spans.1),
            inherent,
            receiver_type,
            body_facts: body.0,
            branch_outcome_candidates: body.1,
            terminal_effect_candidates: body.2,
            continuation_calls: body.3,
            body_unresolved: body.4,
        });
    }
}

fn receiver_shape(receiver: &Receiver) -> String {
    match &receiver.kind {
        ReceiverKind::Value => "self".to_string(),
        ReceiverKind::Reference(_, _, mutability) => {
            if mutability.is_some() {
                "&mut self".to_string()
            } else {
                "&self".to_string()
            }
        }
        ReceiverKind::Typed(_, _) => "self:typed".to_string(),
        _ => "self:unknown".to_string(),
    }
}

impl<'a> SourceClosureVisitor<'a> {
    pub(crate) fn type_shape(ty: &Type) -> String {
        match ty {
            Type::Path(path) => path
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>()
                .join("::"),
            Type::Reference(reference) => format!(
                "&{}{}",
                if reference.mutability.is_some() {
                    "mut "
                } else {
                    ""
                },
                Self::type_shape(&reference.elem)
            ),
            Type::Tuple(tuple) => format!(
                "({})",
                tuple
                    .elems
                    .iter()
                    .map(Self::type_shape)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            _ => "<complex-type>".to_string(),
        }
    }
}
