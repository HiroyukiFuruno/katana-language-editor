use super::super::super::REQUIRED;
use super::{
    EdgeVisitor,
    bindings::{context_imports_for_items, register_typed_binding},
    event::is_paste_path,
    shortcut::input_mut_shortcut_span,
    text_edit::{is_multiline_builder_chain, plain_binding_name},
};
use crate::source_closure::ast_resolution::span_to_text;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    Expr, ExprCall, ExprMethodCall, ExprPath, ImplItemFn, Item, ItemFn, ItemImpl, ItemMod, Macro,
    Pat, PatTupleStruct,
};
const CALL_REQUIRED_COUNT: usize = 4;

impl<'ast> Visit<'ast> for EdgeVisitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.visit_function_body(&node.sig, &node.block, node.span());
    }
    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        let saved_imports = self.context_imports.clone();
        let saved_context = self.context_bindings.clone();
        let saved_non_context = self.non_context_bindings.clone();
        self.context_bindings.clear();
        self.non_context_bindings.clear();
        if let Some((_, items)) = &node.content {
            self.context_imports = context_imports_for_items(items);
        } else {
            self.context_imports.clear();
        }
        self.symbols.push(node.ident.to_string());
        syn::visit::visit_item_mod(self, node);
        self.symbols.pop();
        self.context_imports = saved_imports;
        self.context_bindings = saved_context;
        self.non_context_bindings = saved_non_context;
    }
    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        let saved_impl_context = self.impl_context;
        self.impl_context = match node.self_ty.as_ref() {
            syn::Type::Path(path) if EdgeVisitor::path_symbol(&path.path) == "egui::Context" => {
                Some(true)
            }
            syn::Type::Path(_) => Some(false),
            _ => None,
        };
        syn::visit::visit_item_impl(self, node);
        self.impl_context = saved_impl_context;
    }
    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        self.visit_function_body(&node.sig, &node.block, node.span());
    }

    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        if let Expr::Path(path) = node.func.as_ref() {
            let symbol = EdgeVisitor::path_symbol(&path.path);
            let dependency = REQUIRED[..CALL_REQUIRED_COUNT]
                .iter()
                .find(|required| symbol.ends_with(**required));
            if let Some(dependency) = dependency {
                self.add("call", dependency, node.span());
            }
        } else {
            self.dynamic = true;
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_path(&mut self, node: &'ast ExprPath) {
        if is_paste_path(&node.path) {
            self.add("event", "Event::Paste", node.span());
        }
        syn::visit::visit_expr_path(self, node);
    }

    fn visit_pat_tuple_struct(&mut self, node: &'ast PatTupleStruct) {
        if is_paste_path(&node.path) {
            self.add("event-pattern", "Event::Paste", node.span());
        }
        syn::visit::visit_pat_tuple_struct(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        if node.method == "input_mut" {
            match self.receiver_context(node.receiver.as_ref()) {
                Some(true) => match input_mut_shortcut_span(node) {
                    Ok(span) => {
                        self.add("method", "InputState::consume_shortcut", span);
                        self.shortcut_closure_depth += 1;
                        syn::visit::visit_expr_method_call(self, node);
                        self.shortcut_closure_depth -= 1;
                        return;
                    }
                    Err(reason) => self.receiver_ambiguities.push(format!(
                        "egui::Context::input_mut shortcut provenance rejected: {reason}: {}",
                        span_to_text(&self.file, &node.span())
                    )),
                },
                Some(false) => self.receiver_ambiguities.push(format!(
                    "input_mut receiver is not egui::Context: {}",
                    span_to_text(&self.file, &node.span())
                )),
                None => self.receiver_ambiguities.push(format!(
                    "input_mut receiver provenance is ambiguous: {}",
                    span_to_text(&self.file, &node.span())
                )),
            }
        } else if node.method == "consume_shortcut" && self.shortcut_closure_depth == 0 {
            self.receiver_ambiguities.push(format!(
                "consume_shortcut must be called on the immediate input_mut closure binding: {}",
                span_to_text(&self.file, &node.span())
            ));
        } else if node.method == "show"
            && matches!(node.receiver.as_ref(), Expr::Path(path)
                if path.path.segments.len() == 1
                    && self.text_edit_bindings.get(&EdgeVisitor::path_symbol(&path.path))
                        == Some(&self.closure_depth))
        {
            self.add("method", "TextEdit::show", node.span());
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_local(&mut self, node: &'ast syn::Local) {
        if let Some(name) = plain_binding_name(&node.pat)
            && node
                .init
                .as_ref()
                .is_some_and(|init| is_multiline_builder_chain(init.expr.as_ref()))
        {
            self.text_edit_bindings
                .insert(name.clone(), self.closure_depth);
            self.pending_text_edit_binding = Some(name);
        }
        if let Pat::Type(pat_type) = &node.pat {
            register_typed_binding(
                &pat_type.pat,
                &pat_type.ty,
                &self.context_imports,
                &mut self.context_bindings,
                &mut self.non_context_bindings,
            );
        }
        syn::visit::visit_local(self, node);
    }

    fn visit_expr_closure(&mut self, node: &'ast syn::ExprClosure) {
        let saved_text_edit = self.text_edit_bindings.clone();
        let saved_pending_text_edit = self.pending_text_edit_binding.clone();
        self.closure_depth += 1;
        syn::visit::visit_expr_closure(self, node);
        self.closure_depth -= 1;
        self.text_edit_bindings = saved_text_edit;
        self.pending_text_edit_binding = saved_pending_text_edit;
    }

    fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
        let name = node.ident.to_string();
        self.context_bindings.remove(&name);
        self.non_context_bindings.remove(&name);
        if self.pending_text_edit_binding.as_deref() == Some(name.as_str()) {
            self.pending_text_edit_binding = None;
        } else {
            self.text_edit_bindings.remove(&name);
        }
        syn::visit::visit_pat_ident(self, node);
    }

    fn visit_item(&mut self, node: &'ast Item) {
        match node {
            Item::Struct(item) => {
                self.context_imports.remove(&item.ident.to_string());
            }
            Item::Enum(item) => {
                self.context_imports.remove(&item.ident.to_string());
            }
            Item::Type(item) => {
                self.context_imports.remove(&item.ident.to_string());
            }
            Item::Union(item) => {
                self.context_imports.remove(&item.ident.to_string());
            }
            _ => {}
        }
        syn::visit::visit_item(self, node);
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        self.macro_ambiguity = true;
        syn::visit::visit_macro(self, node);
    }
}
