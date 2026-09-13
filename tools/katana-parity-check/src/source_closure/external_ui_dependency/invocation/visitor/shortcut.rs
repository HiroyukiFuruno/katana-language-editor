use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Expr, ExprClosure, ExprMethodCall, Item, Macro, Pat};

pub(super) fn input_mut_shortcut_span(
    node: &ExprMethodCall,
) -> Result<proc_macro2::Span, &'static str> {
    if node.args.len() != 1 {
        return Err("input_mut requires exactly one closure argument");
    }
    let Some(Expr::Closure(closure)) = node.args.first() else {
        return Err("input_mut closure escapes immediate call");
    };
    let Some(binding) = closure_binding(closure) else {
        return Err("input_mut closure requires one plain binding");
    };
    let mut visitor = ShortcutClosureVisitor {
        binding,
        ..Default::default()
    };
    visitor.visit_expr(&closure.body);
    if visitor.shadowed || visitor.nested_closure || visitor.local_item || visitor.macro_ambiguity {
        return Err("input_mut closure binding provenance is ambiguous");
    }
    if visitor.shortcut_spans.len() != 1 || visitor.other_receiver {
        return Err("input_mut closure must call consume_shortcut once on its binding");
    }
    Ok(visitor.shortcut_spans[0])
}

fn closure_binding(closure: &ExprClosure) -> Option<String> {
    if closure.inputs.len() != 1 {
        return None;
    }
    let Pat::Ident(binding) = closure.inputs.first()? else {
        return None;
    };
    (binding.subpat.is_none()).then(|| binding.ident.to_string())
}

#[derive(Default)]
struct ShortcutClosureVisitor {
    binding: String,
    shortcut_spans: Vec<proc_macro2::Span>,
    shadowed: bool,
    nested_closure: bool,
    local_item: bool,
    macro_ambiguity: bool,
    other_receiver: bool,
}

impl<'ast> Visit<'ast> for ShortcutClosureVisitor {
    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        if node.method == "consume_shortcut" {
            let receiver_is_binding = matches!(node.receiver.as_ref(), Expr::Path(path)
                if path.path.segments.len() == 1 && path.path.is_ident(&self.binding));
            if receiver_is_binding {
                self.shortcut_spans.push(node.span());
            } else {
                self.other_receiver = true;
            }
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
        if node.ident == self.binding {
            self.shadowed = true;
        }
        syn::visit::visit_pat_ident(self, node);
    }

    fn visit_expr_closure(&mut self, _node: &'ast ExprClosure) {
        self.nested_closure = true;
    }

    fn visit_item(&mut self, _node: &'ast Item) {
        self.local_item = true;
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        self.macro_ambiguity = true;
        syn::visit::visit_macro(self, node);
    }
}
