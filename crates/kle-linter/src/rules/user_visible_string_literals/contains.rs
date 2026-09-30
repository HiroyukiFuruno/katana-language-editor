use syn::parse::Parser;
use syn::visit::Visit;

pub(super) fn contains_string_literal(expression: &syn::Expr) -> bool {
    let mut visitor = StringLiteralVisitor::default();
    visitor.visit_expr(expression);
    visitor.found
}

#[derive(Default)]
struct StringLiteralVisitor {
    found: bool,
}

impl<'ast> Visit<'ast> for StringLiteralVisitor {
    fn visit_lit_str(&mut self, _literal: &'ast syn::LitStr) {
        self.found = true;
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        let Some(segment) = mac.path.segments.last() else {
            return;
        };
        if !matches!(
            segment.ident.to_string().as_str(),
            "format" | "format_args" | "concat"
        ) {
            return;
        }
        let Ok(arguments) =
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated
                .parse2(mac.tokens.clone())
        else {
            return;
        };
        for argument in &arguments {
            self.visit_expr(argument);
        }
    }
}
