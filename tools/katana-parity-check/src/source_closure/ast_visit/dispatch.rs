use syn::visit::{self, Visit};
use syn::{
    Block, Expr, ExprCall, ExprClosure, ExprIf, ExprMatch, ExprMethodCall, Macro, Pat, PatOr,
    spanned::Spanned,
};

use super::syntax::{expr_to_syntax, path_resolution_hint, path_to_syntax};

pub(super) fn is_direct_self_receiver(expression: &Expr) -> bool {
    match expression {
        Expr::Path(path) => path.path.is_ident("self"),
        Expr::Field(field) => is_direct_self_receiver(field.base.as_ref()),
        Expr::Index(index) => is_direct_self_receiver(index.expr.as_ref()),
        _ => false,
    }
}

pub(super) fn app_action_pattern_variants(pattern: &Pat) -> Vec<(String, proc_macro2::Span)> {
    match pattern {
        Pat::Or(PatOr { cases, .. }) => {
            cases.iter().flat_map(app_action_pattern_variants).collect()
        }
        Pat::Path(path) => app_action_path_variant(&path.path),
        Pat::TupleStruct(tuple) => app_action_path_variant(&tuple.path),
        Pat::Struct(structure) => app_action_path_variant(&structure.path),
        _ => Vec::new(),
    }
}

fn app_action_path_variant(path: &syn::Path) -> Vec<(String, proc_macro2::Span)> {
    let segments = path.segments.iter().collect::<Vec<_>>();
    if segments.len() == 2 && segments[0].ident == "AppAction" {
        vec![(segments[1].ident.to_string(), segments[1].span())]
    } else {
        Vec::new()
    }
}

struct ArmCallVisitor {
    calls: Vec<String>,
    facts: Vec<super::scan_state::HandlerCallFact>,
    file: String,
    nested: bool,
    unsupported: bool,
    root: std::path::PathBuf,
    current: std::path::PathBuf,
    resolver: Option<super::lexical_resolver::LexicalPathResolver>,
}

impl<'ast> Visit<'ast> for ArmCallVisitor {
    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        match node.func.as_ref() {
            Expr::Path(path) => {
                let syntax = path_to_syntax(&path.path);
                let (path_resolution, path_target) = path_resolution_hint(
                    &path.path,
                    &self.root,
                    &self.current,
                    self.resolver.as_ref(),
                );
                self.calls.push(syntax.clone());
                self.facts.push(super::scan_state::HandlerCallFact {
                    syntax,
                    kind: "path".to_string(),
                    receiver_shape: "static".to_string(),
                    method: None,
                    span: super::ast_resolution::span_to_text(&self.file, &node.span()),
                    path_resolution: Some(path_resolution),
                    path_target,
                });
            }
            _ => self.unsupported = true,
        }
        visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        let syntax = format!("{}.{}", expr_to_syntax(&node.receiver), node.method);
        let self_receiver =
            matches!(node.receiver.as_ref(), Expr::Path(path) if path.path.is_ident("self"));
        self.calls.push(syntax.clone());
        self.facts.push(super::scan_state::HandlerCallFact {
            syntax,
            kind: if self_receiver {
                "self_method"
            } else {
                "method"
            }
            .to_string(),
            receiver_shape: if self_receiver { "self" } else { "other" }.to_string(),
            method: Some(node.method.to_string()),
            span: super::ast_resolution::span_to_text(&self.file, &node.span()),
            path_resolution: None,
            path_target: None,
        });
        if !matches!(node.receiver.as_ref(), Expr::Path(path) if path.path.is_ident("self")) {
            self.unsupported = true;
        }
        visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_closure(&mut self, node: &'ast ExprClosure) {
        self.unsupported = true;
        visit::visit_expr_closure(self, node);
    }

    fn visit_expr_if(&mut self, node: &'ast ExprIf) {
        self.nested = true;
        visit::visit_expr_if(self, node);
    }

    fn visit_expr_match(&mut self, node: &'ast ExprMatch) {
        self.nested = true;
        visit::visit_expr_match(self, node);
    }

    fn visit_macro(&mut self, _node: &'ast Macro) {
        self.unsupported = true;
    }
}

pub(super) fn direct_handler_calls(
    body: &Expr,
    file: &str,
    root: &std::path::Path,
    current: &std::path::Path,
    resolver: Option<&super::lexical_resolver::LexicalPathResolver>,
) -> (
    Vec<String>,
    Vec<super::scan_state::HandlerCallFact>,
    Vec<String>,
) {
    let mut visitor = ArmCallVisitor {
        calls: Vec::new(),
        facts: Vec::new(),
        file: file.to_string(),
        nested: false,
        unsupported: false,
        root: root.to_path_buf(),
        current: current.to_path_buf(),
        resolver: resolver.cloned(),
    };
    visitor.visit_expr(body);
    let mut reasons = Vec::new();
    if visitor.calls.is_empty() {
        reasons.push("zero direct handler calls".to_string());
    }
    if visitor.calls.len() > 1 {
        reasons.push("multiple direct handler calls".to_string());
    }
    if visitor.nested {
        reasons.push("nested condition or match prevents final handler resolution".to_string());
    }
    if visitor.unsupported {
        reasons.push("macro, closure, trait, or indirect dispatch remains unresolved".to_string());
    }
    (visitor.calls, visitor.facts, reasons)
}

pub(super) fn direct_path_calls(block: &Block) -> Vec<String> {
    struct Visitor {
        calls: Vec<String>,
    }

    impl<'ast> Visit<'ast> for Visitor {
        fn visit_expr_call(&mut self, node: &'ast ExprCall) {
            if let Expr::Path(path) = node.func.as_ref() {
                self.calls.push(path_to_syntax(&path.path));
            }
            visit::visit_expr_call(self, node);
        }
    }

    let mut visitor = Visitor { calls: Vec::new() };
    visitor.visit_block(block);
    visitor.calls
}
