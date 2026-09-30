macro_rules! handler_body_visit_calls {
    () => {
    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        match node.func.as_ref() {
            Expr::Path(path) => {
                self.fact("direct_call", path_to_syntax(&path.path), node.span());
                self.candidate(
                    "external_native_boundary",
                    path_to_syntax(&path.path),
                    node.span(),
                    true,
                );
                self.unresolved
                    .push("unresolved import or external-native path call boundary".to_string());
            }
            _ => {
                self.candidate(
                    "dynamic_boundary",
                    expr_to_syntax(node.func.as_ref()),
                    node.span(),
                    true,
                );
                self.unresolved.push("dynamic call boundary".to_string());
            }
        }
        visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        let self_receiver = matches!(
            node.receiver.as_ref(),
            Expr::Path(path) if path.path.is_ident("self")
        );
        self.fact(
            "direct_method_call",
            format!("{}.{}", expr_to_syntax(&node.receiver), node.method),
            node.span(),
        );
        if self_receiver {
            self.continuation_calls
                .push(super::scan_state::ContinuationCallFact {
                    method: node.method.to_string(),
                    syntax: format!("self.{}", node.method),
                    span: super::ast_resolution::span_to_text(&self.file, &node.span()),
                    enclosing_branch_facts: self.branch_stack.clone(),
                    boundary_kinds: self.boundary_stack.clone(),
                });
            self.candidate(
                "same_implementation_continuation_call",
                format!("self.{}", node.method),
                node.span(),
                true,
            );
        } else {
            self.candidate(
                "trait_or_dynamic_boundary",
                format!("{}.{}", expr_to_syntax(&node.receiver), node.method),
                node.span(),
                true,
            );
            self.unresolved
                .push("trait or dynamic receiver call boundary".to_string());
        }
        visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_assign(&mut self, node: &'ast ExprAssign) {
        self.fact(
            "assignment",
            expr_to_syntax(node.left.as_ref()),
            node.span(),
        );
        if is_direct_self_receiver(node.left.as_ref()) {
            self.candidate(
                "direct_self_receiver_assignment",
                expr_to_syntax(node.left.as_ref()),
                node.span(),
                true,
            );
        }
        visit::visit_expr_assign(self, node);
    }

    fn visit_expr_return(&mut self, node: &'ast ExprReturn) {
        self.candidate(
            "return",
            if node.expr.is_some() {
                "early_return"
            } else {
                "return"
            }
            .to_string(),
            node.span(),
            true,
        );
        self.fact(
            "return",
            if node.expr.is_some() {
                "early_return".to_string()
            } else {
                "return".to_string()
            },
            node.span(),
        );
        visit::visit_expr_return(self, node);
    }
    };
}
