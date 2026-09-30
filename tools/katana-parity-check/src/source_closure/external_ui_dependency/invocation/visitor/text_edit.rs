use syn::{Expr, Pat};

use super::EdgeVisitor;

pub(super) fn is_multiline_builder_chain(expr: &Expr) -> bool {
    match expr {
        Expr::Call(call) => matches!(call.func.as_ref(), Expr::Path(path)
            if EdgeVisitor::path_symbol(&path.path).ends_with("TextEdit::multiline")),
        Expr::MethodCall(call) => is_multiline_builder_chain(call.receiver.as_ref()),
        Expr::Group(group) => is_multiline_builder_chain(group.expr.as_ref()),
        Expr::Paren(paren) => is_multiline_builder_chain(paren.expr.as_ref()),
        _ => false,
    }
}

pub(super) fn plain_binding_name(pat: &Pat) -> Option<String> {
    let Pat::Ident(binding) = pat else {
        return None;
    };
    binding.subpat.is_none().then(|| binding.ident.to_string())
}
