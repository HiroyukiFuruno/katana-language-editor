use proc_macro2::Span;
use syn::visit::Visit;
use syn::{Block, FnArg, PatType, Signature};

use super::{EdgeVisitor, bindings::register_typed_binding};
use crate::source_closure::ast_resolution::span_to_text;

impl EdgeVisitor {
    pub(super) fn visit_function_body(&mut self, signature: &Signature, body: &Block, span: Span) {
        if self.function_depth > 0 {
            self.receiver_ambiguities.push(format!(
                "local function body is outside shortcut closure provenance: {}",
                span_to_text(&self.file, &span)
            ));
            return;
        }
        let saved_context = self.context_bindings.clone();
        let saved_non_context = self.non_context_bindings.clone();
        let saved_text_edit = self.text_edit_bindings.clone();
        let saved_pending_text_edit = self.pending_text_edit_binding.clone();
        let saved_imports = self.context_imports.clone();
        self.context_bindings.clear();
        self.non_context_bindings.clear();
        self.text_edit_bindings.clear();
        self.pending_text_edit_binding = None;
        for input in &signature.inputs {
            if let FnArg::Typed(PatType { pat, ty, .. }) = input {
                register_typed_binding(
                    pat,
                    ty,
                    &self.context_imports,
                    &mut self.context_bindings,
                    &mut self.non_context_bindings,
                );
            }
        }
        self.symbols.push(signature.ident.to_string());
        self.function_depth += 1;
        self.visit_block(body);
        self.function_depth -= 1;
        self.symbols.pop();
        self.context_bindings = saved_context;
        self.non_context_bindings = saved_non_context;
        self.text_edit_bindings = saved_text_edit;
        self.pending_text_edit_binding = saved_pending_text_edit;
        self.context_imports = saved_imports;
    }
}
